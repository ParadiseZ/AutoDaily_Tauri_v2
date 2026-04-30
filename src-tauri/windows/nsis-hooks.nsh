!macro NSIS_HOOK_PREUNINSTALL
  MessageBox MB_YESNO|MB_ICONQUESTION "Do you also want to delete AutoDaily user data? This removes local database, settings, logs, scripts, and configured cache folders." IDNO autodaily_skip_user_data_cleanup
    SetOutPath "$PLUGINSDIR"
    File "windows\uninstall-cleanup.ps1"
    nsExec::ExecToLog 'powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$PLUGINSDIR\uninstall-cleanup.ps1"'
  autodaily_skip_user_data_cleanup:
!macroend
