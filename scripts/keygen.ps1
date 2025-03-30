$macAddress = ((Get-NetAdapter | Where-Object { $_.InterfaceType -eq 6 } | Select-Object -First 1 -ExpandProperty MacAddress) -replace "-", "").ToString().ToLower()
$diskInfo = Get-PhysicalDisk | Select-Object -First 1 -Property SerialNumber, Model, FirmwareVersion
$serialNumber = $diskInfo.SerialNumber
$serialNumber = -join $(
    for ($i = 0; $i -lt $serialNumber.Length; $i += 2) {
        if ($i + 1 -lt $serialNumber.Length) {
            $serialNumber[$i + 1], $serialNumber[$i]
        } else {
            $serialNumber[$i]
        }
    }
)

"$macAddress$serialNumber$($diskInfo.Model)$($diskInfo.FirmwareVersion)"