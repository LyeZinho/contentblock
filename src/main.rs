use std::{fs, net::Ipv4Addr, sync::Arc};
use dashmap::DashSet;
use lazy_static::lazy_static;
use trust_dns_server::{
    ServerFuture,
    server::{Request, RequestHandler, ResponseHandler, ResponseInfo},
    authority::MessageResponseBuilder,
    proto::{
        op::{Header, ResponseCode},
        rr::{RData, Record, Name},
    },
};
use trust_dns_resolver::TokioAsyncResolver;
use anyhow::Result;
use tokio::net::UdpSocket;

lazy_static! {
    static ref BLOCKED_DOMAINS: DashSet<String> = {
        let contents = fs::read_to_string("list.txt").unwrap_or_else(|_| String::new());
        contents
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
            .map(|line| {
                // Remove pontos finais e converte para lowercase
                let domain = line.trim().to_lowercase();
                if domain.ends_with('.') {
                    domain[..domain.len()-1].to_string()
                } else {
                    domain
                }
            })
            .collect::<DashSet<_>>()
    };
}

// DNS handler que verifica se o domínio está bloqueado
struct BlockerHandler {
    resolver: Arc<TokioAsyncResolver>,
}

impl BlockerHandler {
    fn new() -> Result<Self> {
        let resolver = TokioAsyncResolver::tokio_from_system_conf()?;
        Ok(Self {
            resolver: Arc::new(resolver),
        })
    }
}

#[async_trait::async_trait]
impl RequestHandler for BlockerHandler {
    async fn handle_request<R: ResponseHandler>(
        &self,
        request: &Request,
        mut response_handle: R,
    ) -> ResponseInfo {
        let query = request.query();
        let mut domain = query.name().to_string().to_lowercase();
        
        // Remove ponto final se existir (FQDN)
        if domain.ends_with('.') {
            domain = domain[..domain.len()-1].to_string();
        }

        println!("🔍 Consultando: {}", domain);

        // Verifica se o domínio ou qualquer subdomínio está bloqueado
        let is_blocked = BLOCKED_DOMAINS.iter().any(|blocked| {
            let blocked_str = blocked.as_str();
            
            // Verifica correspondência exata
            if domain == blocked_str {
                println!("   ➤ Correspondência exata com: {}", blocked_str);
                return true;
            }
            
            // Verifica se é um subdomínio do domínio bloqueado
            if domain.ends_with(&format!(".{}", blocked_str)) {
                println!("   ➤ Subdomínio de: {}", blocked_str);
                return true;
            }
            
            // Verifica se o domínio bloqueado é um subdomínio do consultado
            if blocked_str.ends_with(&format!(".{}", domain)) {
                println!("   ➤ Superdomínio de: {}", blocked_str);
                return true;
            }
            
            false
        });

        if is_blocked {
            println!("🚫 Bloqueado: {}", domain);

            // Retorna 0.0.0.0 para domínios bloqueados
            let record = Record::from_rdata(
                Name::from(query.name().clone()),
                300,
                RData::A(Ipv4Addr::new(0, 0, 0, 0)),
            );

            let builder = MessageResponseBuilder::from_message_request(request);
            let message = builder.build(
                Header::response_from_request(request.header()),
                std::iter::empty(),
                std::iter::once(&record),
                std::iter::empty(),
                std::iter::empty(),
            );

            response_handle.send_response(message).await.unwrap()
        } else {
            // Encaminhar para DNS real
            match self.resolver.lookup_ip(domain.as_str()).await {
                Ok(lookup) => {
                    println!("✅ Permitido: {} -> {}", domain, lookup.iter().next().map(|ip| ip.to_string()).unwrap_or_default());

                    let mut records = Vec::new();
                    for ip in lookup.iter() {
                        if let std::net::IpAddr::V4(ipv4) = ip {
                            records.push(Record::from_rdata(
                                Name::from(query.name().clone()),
                                300,
                                RData::A(ipv4),
                            ));
                        }
                    }

                    let builder = MessageResponseBuilder::from_message_request(request);
                    let message = builder.build(
                        Header::response_from_request(request.header()),
                        std::iter::empty(),
                        records.iter(),
                        std::iter::empty(),
                        std::iter::empty(),
                    );

                    response_handle.send_response(message).await.unwrap()
                }
                Err(e) => {
                    println!("❌ Erro ao resolver {}: {}", domain, e);

                    let mut header = Header::response_from_request(request.header());
                    header.set_response_code(ResponseCode::NXDomain);

                    let builder = MessageResponseBuilder::from_message_request(request);
                    let message = builder.build(
                        header,
                        std::iter::empty(),
                        std::iter::empty(),
                        std::iter::empty(),
                        std::iter::empty(),
                    );

                    response_handle.send_response(message).await.unwrap()
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Iniciando servidor DNS local (porta 53)");
    println!("📝 Carregando lista de domínios bloqueados de list.txt");
    println!("🔢 Domínios carregados: {}", BLOCKED_DOMAINS.len());
    
    // Mostrar alguns exemplos de domínios carregados
    let mut examples: Vec<_> = BLOCKED_DOMAINS.iter().take(5).map(|d| d.clone()).collect();
    examples.sort();
    println!("📋 Exemplos: {:?}", examples);

    let handler = BlockerHandler::new()?;
    let mut server = ServerFuture::new(handler);
    
    let socket = UdpSocket::bind("0.0.0.0:53").await?;
    println!("🎯 Servidor DNS rodando em 0.0.0.0:53");
    
    server.register_socket(socket);
    server.block_until_done().await?;
    
    Ok(())
}