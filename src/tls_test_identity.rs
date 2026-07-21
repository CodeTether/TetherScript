//! Self-signed localhost identity for deterministic TLS tests.

use openssl::asn1::Asn1Time;
use openssl::bn::BigNum;
use openssl::hash::MessageDigest;
use openssl::nid::Nid;
use openssl::pkey::PKey;
use openssl::rsa::Rsa;
use openssl::x509::{
    extension::{BasicConstraints, KeyUsage, SubjectAlternativeName},
    X509NameBuilder, X509,
};

pub(crate) struct TestIdentity {
    pub(crate) certificate: Vec<u8>,
    pub(crate) private_key: Vec<u8>,
}

pub(crate) fn localhost() -> TestIdentity {
    let key = PKey::from_rsa(Rsa::generate(2048).unwrap()).unwrap();
    let mut name = X509NameBuilder::new().unwrap();
    name.append_entry_by_nid(Nid::COMMONNAME, "localhost")
        .unwrap();
    let name = name.build();
    let mut certificate = X509::builder().unwrap();
    certificate.set_version(2).unwrap();
    let serial = BigNum::from_u32(1).unwrap().to_asn1_integer().unwrap();
    certificate.set_serial_number(&serial).unwrap();
    certificate.set_subject_name(&name).unwrap();
    certificate.set_issuer_name(&name).unwrap();
    certificate.set_pubkey(&key).unwrap();
    certificate
        .set_not_before(Asn1Time::days_from_now(0).unwrap().as_ref())
        .unwrap();
    certificate
        .set_not_after(Asn1Time::days_from_now(1).unwrap().as_ref())
        .unwrap();
    let san = SubjectAlternativeName::new()
        .dns("localhost")
        .build(&certificate.x509v3_context(None, None))
        .unwrap();
    certificate.append_extension(san).unwrap();
    certificate
        .append_extension(BasicConstraints::new().critical().ca().build().unwrap())
        .unwrap();
    certificate
        .append_extension(
            KeyUsage::new()
                .digital_signature()
                .key_cert_sign()
                .build()
                .unwrap(),
        )
        .unwrap();
    certificate.sign(&key, MessageDigest::sha256()).unwrap();
    TestIdentity {
        certificate: certificate.build().to_pem().unwrap(),
        private_key: key.private_key_to_pem_pkcs8().unwrap(),
    }
}
