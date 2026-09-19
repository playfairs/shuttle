use crate::error::Result;
use quinn::crypto::rustls::QuicClientConfig;
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName, UnixTime};
use rustls::{DigitallySignedStruct, Error, SignatureScheme};
use std::fmt::Debug;
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Debug)]
struct DevCertificateVerifier;

impl ServerCertVerifier for DevCertificateVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> std::result::Result<ServerCertVerified, Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![
            SignatureScheme::ECDSA_NISTP256_SHA256,
            SignatureScheme::ECDSA_NISTP384_SHA384,
            SignatureScheme::ECDSA_NISTP521_SHA512,
            SignatureScheme::ED25519,
            SignatureScheme::RSA_PSS_SHA256,
            SignatureScheme::RSA_PSS_SHA384,
            SignatureScheme::RSA_PSS_SHA512,
            SignatureScheme::RSA_PKCS1_SHA256,
            SignatureScheme::RSA_PKCS1_SHA384,
            SignatureScheme::RSA_PKCS1_SHA512,
        ]
    }
}

pub async fn create_client_endpoint(_server_addr: SocketAddr) -> Result<(quinn::Endpoint, String)> {
    let rustls_config = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(DevCertificateVerifier))
        .with_no_client_auth();

    let client_config = quinn::ClientConfig::new(Arc::new(
        QuicClientConfig::try_from(rustls_config)
            .map_err(|e| crate::error::ShuttleError::TlsError(e.to_string()))?,
    ));

    let mut endpoint = quinn::Endpoint::new(
        Default::default(),
        None,
        std::net::UdpSocket::bind("[::]:0")
            .or_else(|_| std::net::UdpSocket::bind("127.0.0.1:0"))?,
        std::sync::Arc::new(quinn::TokioRuntime),
    )?;
    endpoint.set_default_client_config(client_config);

    Ok((endpoint, _server_addr.to_string()))
}

pub async fn create_server_endpoint(bind_addr: SocketAddr) -> Result<quinn::Endpoint> {
    let (cert, key) = create_dev_certificates()?;

    let rustls_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert.clone()], key)
        .map_err(|e| crate::error::ShuttleError::TlsError(e.to_string()))?;

    let server_config = quinn::ServerConfig::with_crypto(std::sync::Arc::new(
        quinn::crypto::rustls::QuicServerConfig::try_from(rustls_config)
            .map_err(|e| crate::error::ShuttleError::TlsError(e.to_string()))?,
    ));

    let endpoint = quinn::Endpoint::new(
        Default::default(),
        Some(server_config),
        std::net::UdpSocket::bind(bind_addr)?,
        std::sync::Arc::new(quinn::TokioRuntime),
    )?;

    Ok(endpoint)
}

fn create_dev_certificates() -> Result<(CertificateDer<'static>, PrivateKeyDer<'static>)> {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".to_string()])
        .map_err(|e| crate::error::ShuttleError::TlsError(e.to_string()))?;

    let cert_der = CertificateDer::from(
        cert.serialize_der()
            .map_err(|e| crate::error::ShuttleError::TlsError(e.to_string()))?,
    );

    let key_der = PrivateKeyDer::Pkcs8(rustls::pki_types::PrivatePkcs8KeyDer::from(
        cert.serialize_private_key_der(),
    ));

    Ok((cert_der, key_der))
}
