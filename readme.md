MMBNLC EXE3 Restore Internet Color mod
======================================

This is a mod for Mega Man Battle Network Legacy Collection Vol. 1 which
restores the original Internet colors in MMBN3 after beating the game.
Normally, the Internet's colors change to a muted palette during the final
chapter of the story and stay that way permanently.


Features
--------

 *  Restores the original Internet colors in MMBN3 after beating the game.


Installing
----------

Windows PC and Steam Deck

 1. Download and install chaudloader:
    https://github.com/RockmanEXEZone/chaudloader/releases
    Version 0.11.0 or newer is required.

 2. Launch Steam in Desktop Mode. Right-click the game in Steam, then click
    Properties → Local Files → Browse to open the game's install folder. Then
	open the "exe" folder, where you'll find MMBN_LC1.exe.

 3. Copy the RestoreInternetColor_EXE3 folder to the "mods" folder.

 4. Launch the game as normal.


Version History
---------------

Ver. 1.1.0 - 13 November 2023

 *  Updated for compatibility with latest game update.
 *  chaudloader version 0.11.0 or newer is now required.

Ver. 1.0.0 - 26 October 2023

 *  Initial version.


Building
--------

Building is supported on Windows 10 & 11. First install the following
prerequisites:

 *  Rust

Then, run one of the following commands:

 *  make - Builds the mod files compatible with chaudloader.
 *  make clean - Removes all temporary files and build outputs.
 *  make install - Installs the previously built mod files into the mods folder
    for chaudloader.
 *  make uninstall - Removes the installed mod files from the mods folder for
    chaudloader.
