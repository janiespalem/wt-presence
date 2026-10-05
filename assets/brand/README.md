# Discord artwork

Original radar artwork for the project-owned Discord application `1555607328965926974`. The graphite field, cyan sweep, and amber contact stay consistent across the family. Only the contact geometry changes: diamond for default, ascending chevron for air, grounded block for ground, tapered contact for naval, and open shelter for hangar.

The SVG files in `source/` are the editable masters. All PNG exports are 1024×1024, opaque, 8-bit sRGB. The application icon and default presence asset deliberately share the same mark. The artwork uses four solid colors (`#141B23`, `#2B414E`, `#50E3F2`, `#FFB454`) and no external fonts, images, or dependencies.

## Upload manifest

| Destination | File | Exact Rich Presence key |
| --- | --- | --- |
| Application icon | `wt-presence-icon-1024.png` | — |
| Default / unknown state | `presence-default.png` | `presence-default` |
| Air battle | `presence-air.png` | `presence-air` |
| Ground battle | `presence-ground.png` | `presence-ground` |
| Naval battle | `presence-naval.png` | `presence-naval` |
| Hangar | `presence-hangar.png` | `presence-hangar` |

The five keys match `DiscordIdentity::asset_key` in `src/discord_identity.rs`. Do not use the filenames' `.png` suffix in portal keys or upload them under the obsolete `war_thunder` key.

## Developer Portal setup

The maintainer must perform these steps before release:

1. Open application `1555607328965926974` in the [Discord Developer Portal](https://discord.com/developers/applications/1555607328965926974/information).
2. The application is currently named `12`. Rename it to `WT Presence` and save the change.
3. Set the application icon to `wt-presence-icon-1024.png` and save it.
4. State in the application profile that WT Presence is an unofficial companion for War Thunder and is not affiliated with Gaijin Entertainment.
5. Upload all five PNG files in the Rich Presence assets section using the exact keys in the manifest, and save the changes. Confirm each key is present and its preview matches the file before testing the client.

Local Discord IPC cannot upload these files, rename the application, or inspect the portal's asset catalog. Artwork is implemented; portal upload and manual verification are pending.

## Release verification

On Windows, use Discord Desktop, War Thunder, and a clean WT Presence settings file with no local Application ID setting. Use a clean Discord account for the release check.

1. Launch WT Presence and confirm it requests no Application ID. Check that the activity is named `WT Presence`, its application icon appears, and its current presence asset renders automatically.
2. Check all five uploaded keys through the corresponding states: default while loading or when the battle vehicle domain is unknown, hangar, and air, ground, and naval battles. Record screenshots and confirm the contact geometry matches the manifest.
3. Move from hangar into an air or ground battle. Confirm the asset changes and the elapsed timer continues without resetting.
4. Disable vanilla War Thunder under Discord's Registered Games settings. Confirm only the WT Presence activity remains.
5. Close Discord and confirm WT Presence, telemetry, and the dashboard continue running. Reopen Discord and confirm the activity reconnects without restarting WT Presence.

Keep `docs/design/discord-identity.md` at `implemented; portal verification pending` until the maintainer confirms every check. Record the evidence and change it to `implemented and manually verified` in a separate follow-up commit. The release remains gated on those checks.

## Rebuild exports

From the repository root, use ImageMagick with librsvg support:

```sh
magick assets/brand/source/wt-presence-icon.svg -strip -define png:exclude-chunks=date,time -colorspace sRGB -type TrueColor PNG24:assets/brand/wt-presence-icon-1024.png
for state in default air ground naval hangar; do
  magick "assets/brand/source/presence-$state.svg" -strip -define png:exclude-chunks=date,time -colorspace sRGB -type TrueColor "PNG24:assets/brand/presence-$state.png"
done
identify assets/brand/*.png
```

Every export must report `1024x1024`. Inspect each at full size and 32×32 after changing a master; the amber contacts must remain distinct and the cyan sweep must retain a clear silhouette.
