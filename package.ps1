Copy-Item .\target\release\n-body-problem.exe .\release\n-body-problem.exe
Copy-Item .\assets\*.* .\release\assets\

$sourceFolder = "release"
$destinationZip = "n-body-problem.zip"

Compress-Archive -Path $sourceFolder -DestinationPath $destinationZip -Update