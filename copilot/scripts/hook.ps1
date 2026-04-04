# Copilot CLI hook handler for claude-status-pet
# Called by hooks.json with the event name as the first argument.
param(
    [Parameter(Mandatory)]
    [string]$Event
)

$petBin = "$env:USERPROFILE\.claude\pet-data\bin\claude-status-pet-windows-x64.exe"

& $petBin write-status --adapter copilot --copilot-event $Event
