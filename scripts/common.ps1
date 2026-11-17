# SoloWoW shared helpers (spike). Dot-source this file.
$Bin   = 'C:\SoloWoW\server'
$Mysql = 'C:\Program Files\MySQL\MySQL Server 8.4\bin\mysql.exe'
$Acct  = 'SOLO'; $Pass = 'solo'          # fixed local account for the spike (GM 3, also used for SOAP)
$SoapPort = 7879

function Sql([string]$q, [string]$db = '') {
    $env:MYSQL_PWD = 'acore'
    $a = @('--host=127.0.0.1', '--port=3310', '--user=acore', '--batch', '--skip-column-names', '-e', $q)
    if ($db) { $a += $db }
    & $Mysql @a 2>&1 | Where-Object { "$_" -notmatch '\[Warning\]' }
}
function Test-Port([int]$p) { [bool](Get-NetTCPConnection -LocalPort $p -State Listen -ErrorAction SilentlyContinue) }

function ConvertTo-Hex([byte[]]$b) { -join ($b | ForEach-Object { $_.ToString('X2') }) }
function New-SrpVerifier([string]$user, [string]$pass) {   # same as AzerothManager
    $sha  = [Security.Cryptography.SHA1]::Create()
    $salt = New-Object byte[] 32
    [Security.Cryptography.RandomNumberGenerator]::Create().GetBytes($salt)
    $h1 = $sha.ComputeHash([Text.Encoding]::ASCII.GetBytes($user.ToUpper() + ':' + $pass.ToUpper()))
    $h2 = $sha.ComputeHash([byte[]]($salt + $h1))
    $x  = [bigint]::new([byte[]]($h2 + [byte]0))
    $N  = [bigint]::Parse('00894B645E89E1535BBDAD5B8B290650530801B18EBFBF5E8FAB3C82872A3E9BB7', [Globalization.NumberStyles]::HexNumber)
    $vb = ([bigint]::ModPow([bigint]7, $x, $N)).ToByteArray()
    $out = New-Object byte[] 32; [Array]::Copy($vb, $out, [Math]::Min(32, $vb.Length))
    @{ Salt = $salt; Verifier = $out }
}
function Ensure-Account {
    if (Sql "SELECT id FROM account WHERE username='$Acct'" 'acore_auth') { return }
    $s = New-SrpVerifier $Acct $Pass
    Sql "INSERT INTO account (username, salt, verifier, expansion) VALUES ('$Acct', X'$(ConvertTo-Hex $s.Salt)', X'$(ConvertTo-Hex $s.Verifier)', 2)" 'acore_auth' | Out-Null
    $id = Sql "SELECT id FROM account WHERE username='$Acct'" 'acore_auth'
    Sql "REPLACE INTO account_access (id, gmlevel, RealmID) VALUES ($id, 3, -1)" 'acore_auth' | Out-Null
    Write-Host "Created account $Acct (id $id)"
}
function Invoke-Soap([string]$cmd) {
    $body = '<?xml version="1.0" encoding="utf-8"?><SOAP-ENV:Envelope xmlns:SOAP-ENV="http://schemas.xmlsoap.org/soap/envelope/" xmlns:ns1="urn:AC">' +
            "<SOAP-ENV:Body><ns1:executeCommand><command>$([Security.SecurityElement]::Escape($cmd))</command></ns1:executeCommand></SOAP-ENV:Body></SOAP-ENV:Envelope>"
    $auth = [Convert]::ToBase64String([Text.Encoding]::ASCII.GetBytes("${Acct}:$Pass"))
    try { $r = Invoke-WebRequest "http://127.0.0.1:$SoapPort/" -Method Post -Body $body -ContentType 'text/xml' -Headers @{Authorization="Basic $auth"} -UseBasicParsing -TimeoutSec 600; $c = $r.Content }
    catch [System.Net.WebException] { $c = (New-Object IO.StreamReader($_.Exception.Response.GetResponseStream())).ReadToEnd() }
    ([xml]$c).SelectSingleNode('//*[local-name()="result" or local-name()="faultstring"]').InnerText
}
