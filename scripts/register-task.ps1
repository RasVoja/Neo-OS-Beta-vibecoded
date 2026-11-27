# Registers a Windows Task Scheduler job that runs one NBPOS coding step inside WSL2.
# Run once in PowerShell:  powershell -ExecutionPolicy Bypass -File register-task.ps1
# Registruje zakazan zadatak koji pokrece jedan korak kodiranja u WSL2.
param(
  [string]$Distro = "Ubuntu",
  [string]$Time   = "02:30",          # local time, change as you like
  [string]$Days   = "Monday,Wednesday,Friday,Sunday"
)
$cmd = "bash -lc 'cd ~/nbpos && bash scripts/nightly-step.sh'"
$action  = New-ScheduledTaskAction -Execute "wsl.exe" -Argument "-d $Distro -- $cmd"
$trigger = New-ScheduledTaskTrigger -Weekly -DaysOfWeek $Days.Split(",") -At $Time
$settings = New-ScheduledTaskSettingsSet -StartWhenAvailable -WakeToRun `
  -ExecutionTimeLimit (New-TimeSpan -Hours 5) -MultipleInstances IgnoreNew
Register-ScheduledTask -TaskName "NBPOS nightly step" -Action $action -Trigger $trigger `
  -Settings $settings -Description "Claude Code: one NBPOS step, own branch + PR" -Force
Write-Host "Registered. Test now:  Start-ScheduledTask -TaskName 'NBPOS nightly step'"
