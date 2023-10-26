@echo off
setlocal enabledelayedexpansion

rem Mod info
set "MOD_DIR=RestoreInternetColor_EXE3"

rem Install locations
set "VOL1_DIR=C:\Program Files (x86)\Steam\steamapps\common\MegaMan_BattleNetwork_LegacyCollection_Vol1"

rem Build folder
set "BUILD_DIR=_build"
set "BUILD_DIR_VOL1=!BUILD_DIR!\!MOD_DIR!"
set "INSTALL_DIR_VOL1=!VOL1_DIR!\exe\mods\!MOD_DIR!"

if /I [%1]==[clean] (
	echo Running cargo clean...
	cargo clean ^
		1> nul || goto :error
	echo Removing build folder...
	if exist "!BUILD_DIR!" (
		rmdir /S /Q "!BUILD_DIR!" ^
			1> nul || goto :error
	)
	echo.

	goto :done
)

if /I [%1]==[install] (
	if exist "!VOL1_DIR!" (
		if exist "!BUILD_DIR_VOL1!" (
			echo Installing for Volume 1...

			echo Copying mod folder...
			if exist "!INSTALL_DIR_VOL1!" (
				del /F /S /Q "!INSTALL_DIR_VOL1!\*" 1> nul || goto :error
			) else (
				mkdir "!INSTALL_DIR_VOL1!" 1> nul || goto :error
			)
			robocopy /E "!BUILD_DIR_VOL1!" "!INSTALL_DIR_VOL1!" 1> nul
			if errorlevel 8 goto :error
		) else (
			echo Volume 1 not built; skipping...
		)
	) else (
		echo Volume 1 not installed; skipping...
	)
	echo.

	goto :done
)

if /I [%1]==[uninstall] (
	if exist "!VOL1_DIR!" (
		echo Uninstalling for Volume 1...
		if exist "!INSTALL_DIR_VOL1!" (
			rmdir /S /Q "!INSTALL_DIR_VOL1!" ^
				1> nul || goto :error
		)
	) else (
		echo Volume 1 not installed; skipping...
	)
	echo.

	goto :done
)

rem Build mod
echo Building for Volume 1...

rem Clean build folder
call :clean_folder "!BUILD_DIR_VOL1!"

echo Running cargo build...
cargo build --release ^
	1> nul || goto :error

echo Copying mod files...
copy "target\release\patch.dll" "!BUILD_DIR_VOL1!" ^
	1> nul || goto :error
copy /Y "info.toml" "!BUILD_DIR_VOL1!\info.toml" ^
	1> nul || goto :error
copy /Y "init.lua" "!BUILD_DIR_VOL1!\init.lua" ^
	1> nul || goto :error
echo.

rem Copy miscellaneous files
copy /Y "readme.md" "!BUILD_DIR!\readme.txt" ^
	1> nul || goto :error

:done
echo Done.
echo.
exit /b 0

:error
echo Error occurred, failed to build.
echo.
exit /b 1

:clean_folder
if exist "%1" (
	del /F /S /Q "%1\*" 1> nul || goto :error
) else (
	mkdir "%1" 1> nul || goto :error
)
