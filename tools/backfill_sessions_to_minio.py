import base64
import datetime as dt
import hashlib
import hmac
import json
import os
import re
import socket
import subprocess
import sys
import urllib.error
import urllib.parse
import urllib.request

MAX_OBJECT = 8 * 1024 * 1024
BUCKET = "codetether-training"
NAMESPACE = "codetether-data"
SECRET = "training-data-secrets"
HOME = os.path.expanduser("~")
HOST = re.sub(r"[^A-Za-z0-9_.-]+", "-", socket.gethostname()).strip("-")
SENDER = f"machine-{HOST}"
ROOT_PREFIX = "training/v2"


def utc_now():
    return dt.datetime.now(dt.timezone.utc).strftime("%Y%m%dT%H%M%SZ")


def parse_time(value):
    if not value:
        return dt.datetime.fromtimestamp(0, dt.timezone.utc)
    try:
        out = dt.datetime.fromisoformat(str(value).replace("Z", "+00:00"))
        return out if out.tzinfo else out.replace(tzinfo=dt.timezone.utc)
    except ValueError:
        return dt.datetime.fromtimestamp(0, dt.timezone.utc)


def load_secret():
    cmd = ["kubectl", "get", "secret", "-n", NAMESPACE, SECRET, "-o", "json"]
    proc = subprocess.run(cmd, text=True, capture_output=True, check=False)
    if proc.returncode != 0:
        raise RuntimeError("failed to read Kubernetes MinIO secret")
    data = json.loads(proc.stdout).get("data", {})

    def dec(name):
        if name not in data:
            raise RuntimeError(f"secret missing {name}")
        return base64.b64decode(data[name]).decode("utf-8")

    return dec("minio-endpoint").rstrip("/"), dec("minio-access-key"), dec("minio-secret-key")


def sign(key, msg):
    return hmac.new(key, msg.encode("utf-8"), hashlib.sha256).digest()


def aws_key(secret, date):
    k_date = sign(("AWS4" + secret).encode("utf-8"), date)
    k_region = sign(k_date, "us-east-1")
    k_service = sign(k_region, "s3")
    return sign(k_service, "aws4_request")


def s3_request(endpoint, access, secret, method, key, body=b"", extra=None):
    parsed = urllib.parse.urlparse(endpoint)
    quoted = "/".join(urllib.parse.quote(p, safe="-_.~/") for p in key.split("/"))
    path = f"/{BUCKET}/{quoted}"
    url = urllib.parse.urlunparse((parsed.scheme, parsed.netloc, path, "", "", ""))
    now = dt.datetime.now(dt.timezone.utc)
    amz_date = now.strftime("%Y%m%dT%H%M%SZ")
    date_stamp = now.strftime("%Y%m%d")
    payload_hash = hashlib.sha256(body).hexdigest()
    headers = {
        "host": parsed.netloc,
        "x-amz-content-sha256": payload_hash,
        "x-amz-date": amz_date,
    }
    if extra:
        headers.update({k.lower(): v for k, v in extra.items()})
    names = sorted(headers)
    canonical_headers = "".join(f"{h}:{str(headers[h]).strip()}\n" for h in names)
    canonical = "\n".join([method, path, "", canonical_headers, ";".join(names), payload_hash])
    scope = f"{date_stamp}/us-east-1/s3/aws4_request"
    digest = hashlib.sha256(canonical.encode()).hexdigest()
    to_sign = "\n".join(["AWS4-HMAC-SHA256", amz_date, scope, digest])
    sig = hmac.new(aws_key(secret, date_stamp), to_sign.encode(), hashlib.sha256).hexdigest()
    headers["authorization"] = (
        f"AWS4-HMAC-SHA256 Credential={access}/{scope}, "
        f"SignedHeaders={';'.join(names)}, Signature={sig}"
    )
    req = urllib.request.Request(url, data=body if method != "HEAD" else None, headers=headers, method=method)
    try:
        with urllib.request.urlopen(req, timeout=30) as resp:
            return resp.status, dict(resp.headers)
    except urllib.error.HTTPError as err:
        return err.code, dict(err.headers)


def is_reparse(entry):
    attrs = getattr(entry.stat(follow_symlinks=False), "st_file_attributes", 0)
    return bool(attrs & 0x400)


def find_session_files():
    found = []
    stack = [HOME]
    while stack:
        root = stack.pop()
        try:
            entries = list(os.scandir(root))
        except OSError:
            continue
        if os.path.basename(root) == "sessions" and os.path.basename(os.path.dirname(root)) == ".codetether-agent":
            found.extend(
                e.path for e in entries if e.is_file(follow_symlinks=False) and e.name.endswith(".json")
            )
            continue
        for entry in entries:
            try:
                if entry.is_dir(follow_symlinks=False) and not is_reparse(entry):
                    stack.append(entry.path)
            except OSError:
                pass
    return found


def canonical_sessions(paths):
    chosen = {}
    dupes = 0
    errors = 0
    for path in paths:
        try:
            raw = open(path, "rb").read()
            data = json.loads(raw.decode("utf-8"))
        except Exception:
            errors += 1
            continue
        sid = str(data.get("id") or os.path.splitext(os.path.basename(path))[0])
        score = (parse_time(data.get("updated_at")), len(data.get("messages") or []), len(raw))
        if sid in chosen:
            dupes += 1
            if score <= chosen[sid]["score"]:
                continue
        chosen[sid] = {"path": path, "data": data, "score": score}
    return chosen, dupes, errors


def tool_ids(message):
    ids = []
    content = message.get("content")
    parts = content if isinstance(content, list) else []
    for part in parts:
        if not isinstance(part, dict):
            continue
        for name in ("id", "tool_call_id"):
            value = part.get(name)
            if value and value not in ids:
                ids.append(value)
    return ids


def session_jsonl(sid, data, path):
    keep = {"developer", "system", "user", "assistant", "tool"}
    lines = []
    for i, msg in enumerate(data.get("messages") or []):
        role = msg.get("role")
        if role not in keep:
            continue
        record = {
            "schema": "training/v2",
            "correlation": sid,
            "sender": SENDER,
            "role": role,
            "content": msg.get("content"),
            "tool_ids": tool_ids(msg),
            "metadata": {
                "source": "codetether-session",
                "session_id": sid,
                "message_index": i,
                "session_created_at": data.get("created_at"),
                "session_updated_at": data.get("updated_at"),
                "source_path_sha256": hashlib.sha256(path.encode()).hexdigest(),
            },
        }
        lines.append(json.dumps(record, ensure_ascii=False, separators=(",", ":")))
    return ("\n".join(lines) + ("\n" if lines else "")).encode("utf-8")


def session_key(data, sid):
    timestamp = parse_time(data.get("updated_at") or data.get("created_at"))
    return f"{ROOT_PREFIX}/{timestamp:%Y/%m/%d/%H}/machine_{HOST}_session_{sid}.jsonl"


def pending_files():
    root = os.path.join(HOME, ".codetether", "training", "pending")
    out = []
    if not os.path.isdir(root):
        return out
    for dirpath, _, names in os.walk(root):
        for name in names:
            path = os.path.join(dirpath, name)
            rel = os.path.relpath(path, root).replace(os.sep, "/")
            if rel.startswith(ROOT_PREFIX + "/"):
                out.append((rel, path))
    return out


def head_then_put(endpoint, access, secret, key, body, ev):
    sha = hashlib.sha256(body).hexdigest()
    if len(body) > MAX_OBJECT:
        ev["oversized"] += 1
        ev["oversized_keys"].append({"key": key, "bytes": len(body)})
        return
    status, headers = s3_request(endpoint, access, secret, "HEAD", key)
    ev["head_count"] += 1
    if status == 200:
        remote = headers.get("x-amz-meta-sha256") or headers.get("X-Amz-Meta-Sha256")
        if remote == sha:
            ev["skipped_existing_same_sha"] += 1
        else:
            ev["conflicts"] += 1
            ev["conflict_keys"].append({"key": key, "local_sha256": sha, "remote_sha256": remote or "missing"})
        return
    if status not in (403, 404):
        ev["head_errors"] += 1
        ev["errors"].append({"key": key, "stage": "HEAD", "status": status})
        return
    extra = {"content-type": "application/x-ndjson", "x-amz-meta-sha256": sha}
    status, _ = s3_request(endpoint, access, secret, "PUT", key, body, extra)
    if 200 <= status < 300:
        ev["uploaded"] += 1
        ev["uploaded_bytes"] += len(body)
    else:
        ev["put_errors"] += 1
        ev["errors"].append({"key": key, "stage": "PUT", "status": status})


def main():
    ev = {
        "validation_level": "MinIO",
        "started_at": utc_now(),
        "sender": SENDER,
        "bucket": BUCKET,
        "session_source_files": 0,
        "canonical_sessions": 0,
        "duplicate_session_files": 0,
        "session_parse_errors": 0,
        "pending_files": 0,
        "head_count": 0,
        "uploaded": 0,
        "uploaded_bytes": 0,
        "skipped_existing_same_sha": 0,
        "conflicts": 0,
        "conflict_keys": [],
        "oversized": 0,
        "oversized_keys": [],
        "head_errors": 0,
        "put_errors": 0,
        "errors": [],
    }
    endpoint, access, secret = load_secret()
    session_paths = find_session_files()
    sessions, dupes, parse_errors = canonical_sessions(session_paths)
    ev.update({
        "session_source_files": len(session_paths),
        "canonical_sessions": len(sessions),
        "duplicate_session_files": dupes,
        "session_parse_errors": parse_errors,
    })
    for sid in sorted(sessions):
        item = sessions[sid]
        body = session_jsonl(sid, item["data"], item["path"])
        head_then_put(endpoint, access, secret, session_key(item["data"], sid), body, ev)
    pending = pending_files()
    ev["pending_files"] = len(pending)
    for key, path in pending:
        with open(path, "rb") as handle:
            head_then_put(endpoint, access, secret, key, handle.read(), ev)
    ev["finished_at"] = utc_now()
    evidence_dir = os.path.join(HOME, ".codetether", "training", "evidence")
    os.makedirs(evidence_dir, exist_ok=True)
    out = os.path.join(evidence_dir, f"minio_backfill_{HOST}_{utc_now()}.json")
    with open(out, "w", encoding="utf-8") as handle:
        json.dump(ev, handle, indent=2, sort_keys=True)
    keys = ["validation_level", "sender", "bucket", "session_source_files", "canonical_sessions", "pending_files", "uploaded", "uploaded_bytes", "skipped_existing_same_sha", "conflicts", "oversized", "head_errors", "put_errors"]
    print(json.dumps({key: ev[key] for key in keys}, sort_keys=True))
    print("evidence_path", out)
    return 1 if ev["errors"] or ev["oversized"] else 0


if __name__ == "__main__":
    sys.exit(main())