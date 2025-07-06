# ------------------------------------------
# setup.ps1 - Instala servidor DNS Blocker
# como serviço no Windows (requer admin)
# ------------------------------------------

# 🚨 Verifica permissões de admin
if (-not ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")) {
    Write-Warning "Execute este script como ADMINISTRADOR!"
    exit 1
}

# 📁 Caminho do executável do DNS Blocker
$dnsBlockerPath = "E:\git\contentblock\target\release\domain_blocker.exe"

# 🔧 Verifica se o executável existe
if (-Not (Test-Path $dnsBlockerPath)) {
    Write-Error "❌ Executável não encontrado em: $dnsBlockerPath"
    Write-Host "➡️ Copie seu 'domain_blocker.exe' para esse caminho ou edite o script."
    exit 1
}

# ✅ Detecta interface ativa (Wi-Fi ou Ethernet)
$interface = Get-NetAdapter | Where-Object { $_.Status -eq 'Up' } | Select-Object -First 1
if (-not $interface) {
    Write-Error "❌ Nenhuma interface de rede ativa encontrada!"
    exit 1
}

Write-Host "🔍 Interface detectada: $($interface.Name)"

# 🔐 Permite tráfego na porta 53 UDP
Write-Host "🛡️  Configurando firewall..."
New-NetFirewallRule -DisplayName "Allow DNS Blocker" `
    -Direction Inbound -Protocol UDP -LocalPort 53 -Action Allow -Profile Any -Enabled True -ErrorAction SilentlyContinue

# 🌐 Configura DNS local para 127.0.0.1
Write-Host "🔧 Configurando DNS para 127.0.0.1..."
Set-DnsClientServerAddress -InterfaceAlias $interface.Name -ServerAddresses ("127.0.0.1")

# 🧰 Instala como serviço usando sc.exe
$serviceName = "DomainBlockerDNS"
Write-Host "⚙️  Registrando serviço '$serviceName'..."
sc.exe create $serviceName binPath= "`"$dnsBlockerPath`"" start= auto
sc.exe description $serviceName "DNS Blocker Service (intercepta domínios localmente)"
sc.exe start $serviceName

# ✅ Concluído
Write-Host "`n✅ Serviço '$serviceName' instalado e iniciado!"
Write-Host "🌐 DNS agora aponta para 127.0.0.1"
Write-Host "📦 Binário usado: $dnsBlockerPath"
