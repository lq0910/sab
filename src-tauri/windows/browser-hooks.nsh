; 功能：安装时注册当前用户的浏览器候选和URL命令，卸载清理自己的注册项；时间：2026-11-14；作者：lq。
; Windows 10/11的默认选择由系统设置确认，禁止直接写UserChoice。
!macro NSIS_HOOK_POSTINSTALL
  WriteRegStr HKCU "Software\Classes\SAB.Url" "" "轻页 SAB 网页链接"
  WriteRegStr HKCU "Software\Classes\SAB.Url" "URL Protocol" ""
  WriteRegStr HKCU "Software\Classes\SAB.Url\DefaultIcon" "" '$"$INSTDIR\sab.exe$",0'
  WriteRegStr HKCU "Software\Classes\SAB.Url\shell\open\command" "" '$"$INSTDIR\sab.exe$" $"%1$"'
  WriteRegStr HKCU "Software\Clients\StartMenuInternet\SAB" "" "轻页 SAB"
  WriteRegStr HKCU "Software\Clients\StartMenuInternet\SAB\DefaultIcon" "" '$"$INSTDIR\sab.exe$",0'
  WriteRegStr HKCU "Software\Clients\StartMenuInternet\SAB\shell\open\command" "" '$"$INSTDIR\sab.exe$"'
  WriteRegStr HKCU "Software\Clients\StartMenuInternet\SAB\Capabilities" "ApplicationName" "轻页 SAB"
  WriteRegStr HKCU "Software\Clients\StartMenuInternet\SAB\Capabilities" "ApplicationDescription" "轻量网页浏览与文件下载"
  WriteRegStr HKCU "Software\Clients\StartMenuInternet\SAB\Capabilities" "ApplicationIcon" '$"$INSTDIR\sab.exe$",0'
  WriteRegStr HKCU "Software\Clients\StartMenuInternet\SAB\Capabilities\URLAssociations" "http" "SAB.Url"
  WriteRegStr HKCU "Software\Clients\StartMenuInternet\SAB\Capabilities\URLAssociations" "https" "SAB.Url"
  WriteRegStr HKCU "Software\RegisteredApplications" "轻页 SAB" "Software\Clients\StartMenuInternet\SAB\Capabilities"
!macroend

; 功能：仅当命令仍指向本安装目录才清理，避免旧卸载器移除新安装的候选注册；时间：2026-11-14；作者：lq。
!macro NSIS_HOOK_PREUNINSTALL
  ReadRegStr $R0 HKCU "Software\Classes\SAB.Url\shell\open\command" ""
  ${If} $R0 == '$"$INSTDIR\sab.exe$" $"%1$"'
    DeleteRegValue HKCU "Software\RegisteredApplications" "轻页 SAB"
    DeleteRegKey HKCU "Software\Clients\StartMenuInternet\SAB"
    DeleteRegKey HKCU "Software\Classes\SAB.Url"
  ${EndIf}
!macroend
