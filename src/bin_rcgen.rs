fn main() {
    let cert_params = rcgen::CertificateParams::new(vec!["fastshare.local".into()]).unwrap();
    let key_pair = rcgen::KeyPair::generate().unwrap();
    let cert = cert_params.self_signed(&key_pair).unwrap();
    let cert_der = cert.der();
    let key_der = key_pair.serialize_der();
}
