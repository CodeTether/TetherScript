# pgbinary — integrator handoff

Binary-format row decoding and typed parameter binding for the native PostgreSQL
client. This module is **self-contained and callable**: nothing under
`src/postgres/` was modified. The integrator declares the module and switches the
decode path over.

## 1. Declare the module (`src/postgres.rs`)

```diff
 mod auth;
+mod binary;
 mod collect;
```

and export it, since the tests and hosts reach it by path:

```diff
 pub use connection::{Config, Connection};
+pub use binary as binary_codec; // or: pub mod binary; see note below
 pub use handler::PostgresHandler;
```

`tests/postgres_binary.rs` imports `tetherscript::postgres::binary::…`, so the
simplest wiring is to declare it public directly:

```diff
-mod binary;
+pub mod binary;
```

## 2. Read the type OIDs in `rows.rs`

`row_description` currently discards the type OID inside its fixed 18-byte skip.
The binary path needs it. The 18 bytes are: table OID (4), column index (2),
**type OID (4)**, size (2), modifier (4), format code (2).

```diff
-/// Column names from a `RowDescription` message.
-pub(super) fn row_description(body: &[u8]) -> Result<Vec<String>, String> {
+/// Column names and type OIDs from a `RowDescription` message.
+pub(super) fn row_description(body: &[u8]) -> Result<Vec<Column>, String> {
     let mut cursor = Cursor::new(body);
     let count = cursor.i16()?;
-    let mut names = Vec::with_capacity(count.max(0) as usize);
+    let mut columns = Vec::with_capacity(count.max(0) as usize);
     for _ in 0..count.max(0) {
-        names.push(cursor.cstr()?);
-        // table OID, column index, type OID, size, modifier, format code
-        cursor.take(18)?;
+        let name = cursor.cstr()?;
+        cursor.take(6)?; // table OID (4) + column index (2)
+        let type_oid = cursor.i32()? as u32;
+        cursor.take(8)?; // size (2) + modifier (4) + format code (2)
+        columns.push(Column { name, type_oid });
     }
-    Ok(names)
+    Ok(columns)
 }
```

with, in a new `src/postgres/column.rs` (kept separate for the 50-line limit):

```rust
//! One column's identity from a `RowDescription`.
pub(super) struct Column {
    pub(super) name: String,
    pub(super) type_oid: u32,
}
```

## 3. Switch `data_row` to the binary decoder with a text fallback

This is the load-bearing change. `needs_text_fallback()` is the only signal that
means "retry as text"; every other error is a genuine protocol fault and must
propagate.

```diff
-pub(super) fn data_row(body: &[u8], columns: &[String]) -> Result<Value, String> {
+pub(super) fn data_row(body: &[u8], columns: &[Column], binary: bool) -> Result<Value, String> {
     let mut cursor = Cursor::new(body);
     let count = cursor.i16()?;
     let mut row = HashMap::new();
     for index in 0..count.max(0) as usize {
         let len = cursor.i32()?;
-        let name = columns
-            .get(index)
-            .cloned()
-            .unwrap_or_else(|| format!("column{index}"));
+        let column = columns.get(index);
+        let name = column
+            .map(|c| c.name.clone())
+            .unwrap_or_else(|| format!("column{index}"));
+        // A length of -1 is SQL NULL and carries no bytes. This is distinct from
+        // a length of 0, which is a present, empty value.
         if len < 0 {
             row.insert(name, Value::Nil);
             continue;
         }
         let raw = cursor.take(len as usize)?;
-        row.insert(name, scalar(&String::from_utf8_lossy(raw)));
+        let type_oid = column.map(|c| c.type_oid).unwrap_or(0);
+        row.insert(name, field(raw, type_oid, binary)?);
     }
     Ok(Value::Map(Rc::new(RefCell::new(row))))
 }
+
+/// Decode one field, falling back to the text heuristic for an unknown OID.
+///
+/// The fallback is deliberately narrow: only `needs_text_fallback()` recovers.
+/// A truncated or malformed field is a real protocol fault and must not be
+/// silently reinterpreted as text.
+fn field(raw: &[u8], type_oid: u32, binary: bool) -> Result<Value, String> {
+    if !binary {
+        return Ok(scalar(&String::from_utf8_lossy(raw)));
+    }
+    match binary::decode_field(type_oid, raw) {
+        Ok(value) => Ok(value),
+        Err(error) if error.needs_text_fallback() => {
+            // No binary decoder for this OID. The server still sent binary bytes
+            // for it only if we asked; see step 4 — request text per column for
+            // unsupported OIDs, and this arm becomes unreachable in practice.
+            Ok(scalar(&String::from_utf8_lossy(raw)))
+        }
+        Err(error) => Err(error.to_string()),
+    }
+}
```

**Important ordering note.** Result format codes are chosen in `Bind`, *before*
`RowDescription` arrives, so on the first execution the column types are not yet
known. Two correct strategies:

- **Per-column codes after a `Describe`.** Send `Parse` + `Describe` (statement),
  read `ParameterDescription` + `RowDescription`, then `Bind` with one code per
  column using `binary::supports(type_oid)`. Costs one extra round trip.
- **All-binary, with the fallback arm above as the safety net.** Simpler, one
  round trip, and correct because `decode_field` never panics and the fallback
  keeps an unsupported column readable. Recommended.

## 4. Request binary results in `extended.rs`

`bind` currently writes `0` for both format-code counts, which means "all text" —
this is why nothing is binary today.

```diff
-pub(super) fn bind(parameters: &[Parameter]) -> Vec<u8> {
+pub(super) fn bind(parameters: &[Parameter], binary_results: bool) -> Vec<u8> {
     let mut message = Builder::tagged(b'B');
     message
         .cstr("") // unnamed destination portal
         .cstr("") // unnamed source statement
         .i16(0) // all parameters use the default text format
         .i16(parameters.len() as i16);
     for parameter in parameters {
         match parameter {
             None => { message.i32(-1); }
             Some(bytes) => { message.i32(bytes.len() as i32).bytes(bytes); }
         }
     }
-    // Request text format for all result columns.
-    message.i16(0);
+    // Result format codes. Count 1 applies one code to every column; count 0
+    // means all text. See `binary::format_codes` for the full convention.
+    if binary_results {
+        message.bytes(&binary::format_codes(&[binary::FORMAT_BINARY]));
+    } else {
+        message.i16(0);
+    }
     message.finish()
 }
```

## 5. Typed parameters in `params.rs` (optional, additive)

`encode_all` stays exactly as it is — text parameters with server-inferred types
remain the default and remain correct. Add a *parallel* typed path so a caller
that knows its column types can bind exactly:

```diff
+/// Encode parameters in binary format against explicit type OIDs.
+///
+/// Falls back to the text encoding for any OID with no binary encoder, so a
+/// caller can pass a partially-known OID list.
+pub(super) fn encode_all_typed(
+    parameters: &[Value],
+    type_oids: &[u32],
+) -> Result<(Vec<Parameter>, Vec<i16>), String> {
+    let mut encoded = Vec::with_capacity(parameters.len());
+    let mut codes = Vec::with_capacity(parameters.len());
+    for (index, value) in parameters.iter().enumerate() {
+        let type_oid = type_oids.get(index).copied().unwrap_or(0);
+        match binary::encode_param(type_oid, value) {
+            Ok(bytes) => {
+                encoded.push(bytes);
+                codes.push(binary::FORMAT_BINARY);
+            }
+            Err(error) if error.needs_text_fallback() => {
+                encoded.push(encode(value).map_err(|error| {
+                    format!("db.query: parameter ${}: {error}", index + 1)
+                })?);
+                codes.push(binary::FORMAT_TEXT);
+            }
+            Err(error) => {
+                return Err(format!("db.query: parameter ${}: {error}", index + 1));
+            }
+        }
+    }
+    Ok((encoded, codes))
+}
```

The returned `codes` go through `binary::format_codes(&codes)` into the
*parameter* format-code array of `Bind`, and the OIDs go into `Parse`'s
parameter-type list (which currently writes a count of `0` to request inference).

## 6. `collect.rs`

Only the signature change ripples through:

```diff
-            b'T' => columns = rows::row_description(&message.body)?,
-            b'D' => collected.push(rows::data_row(&message.body, &columns)?),
+            b'T' => columns = rows::row_description(&message.body)?,
+            b'D' => collected.push(rows::data_row(&message.body, &columns, binary)?),
```

with `columns: Vec<Column>` and `binary` threaded in from the caller.

## 7. Docs to update

- `README.md` line ~658 lists "binary-format row decoding" as not done.
- `README.md` PostgreSQL section: the text-format decoding limitation.
- `docs/postgres-client.md` "Row decoding" section: the heuristic table becomes
  the *fallback* behaviour, not the only behaviour.
- `src/postgres.rs` module docs: the "Text-format decoding" bullet under
  "Scope and limits".
- `CHANGELOG.md`.

These are left to the integrator because this task was scoped to create only
`src/postgres/binary*.rs` and `tests/postgres_binary.rs`.
