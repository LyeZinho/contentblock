# Domain Blocker DNS Server

Um servidor DNS local em Rust que bloqueia domínios indesejados redirecionando-os para 0.0.0.0.

## ✨ Funcionalidades

- 🚫 Bloqueia domínios baseado em uma lista (`list.txt`)
- 🔄 Encaminha consultas legítimas para resolvers DNS reais
- ⚡ Alta performance usando Rust e async/await
- 🛡️ Intercepta consultas DNS na porta 53
- 📝 Log detalhado de consultas bloqueadas e permitidas

## 🚀 Como usar

### 1. Compilar o projeto
```bash
cargo build --release
```

### 2. Instalar como serviço (Windows)
Execute o PowerShell como **Administrador** e rode:
```powershell
.\setup_dns.ps1
```

O script irá:
- ✅ Verificar permissões de administrador
- 🔧 Configurar firewall para porta 53 UDP
- 🌐 Alterar DNS da interface ativa para 127.0.0.1
- ⚙️ Instalar e iniciar o serviço "DomainBlockerDNS"

### 3. Gerenciar o serviço
```powershell
# Parar o serviço
sc.exe stop DomainBlockerDNS

# Iniciar o serviço
sc.exe start DomainBlockerDNS

# Remover o serviço
sc.exe delete DomainBlockerDNS
```

### 4. Restaurar DNS original
Para voltar ao DNS original:
```powershell
# Listar interfaces
Get-NetAdapter | Where-Object { $_.Status -eq 'Up' }

# Restaurar DNS automático (substitua "Wi-Fi" pelo nome da sua interface)
Set-DnsClientServerAddress -InterfaceAlias "Wi-Fi" -ResetServerAddresses
```

## 📝 Lista de domínios

O arquivo `list.txt` contém a lista de domínios bloqueados. Você pode:
- Adicionar novos domínios (um por linha)
- Comentar linhas com `#`
- Usar subdomínios (ex: `example.com` bloqueia `www.example.com`)

## 🔍 Como funciona

1. **Consulta DNS recebida** → Servidor DNS local (porta 53)
2. **Verificação** → Domínio está na lista de bloqueio?
   - ✅ **SIM**: Retorna 0.0.0.0 (bloqueado)
   - ❌ **NÃO**: Encaminha para DNS real (ex: 1.1.1.1)
3. **Resposta** → Cliente recebe IP válido ou bloqueio

## 🛠️ Dependências

- `trust-dns-server` - Servidor DNS
- `trust-dns-resolver` - Resolver DNS para encaminhamento
- `trust-dns-client` - Cliente DNS
- `tokio` - Runtime async
- `dashmap` - HashMap thread-safe
- `lazy_static` - Carregamento único da lista
- `anyhow` - Tratamento de erros

## ⚠️ Requisitos

- **Windows**: Administrador (para porta 53 e serviço)
- **Rust**: 1.70+ com features tokio completas
- **Firewall**: Porta 53 UDP liberada

## 📊 Logs

O servidor mostra em tempo real:
```
🚀 Iniciando servidor DNS local (porta 53)
📝 Carregando lista de domínios bloqueados de list.txt
🔢 Domínios carregados: 13185
🎯 Servidor DNS rodando em 0.0.0.0:53
🔍 Consultando: google.com
✅ Permitido: google.com -> 142.250.191.14
🔍 Consultando: exemplo-bloqueado.com
🚫 Bloqueado: exemplo-bloqueado.com
```

## 🔧 Configuração avançada

Para usar DNS específicos, edite o código em `src/main.rs`:
```rust
// Linha ~89 - trocar o resolver
let resolver = TokioAsyncResolver::tokio(
    ResolverConfig::cloudflare(), // ou google(), quad9(), etc.
    ResolverOpts::default()
)?;
```
