## VFS

No VFS! that's too complicated. Instead, just use symlinks/hardlinks. (I hope symlinks alone will work)
So ig on boot it symlinks all the main files and then goes through the filesystem data (see below in data>filesystem) and chooses highest priority or patch.

## Configuration

Where is the source
What command
What does your "Action" do
Symlink / Hardlink settings

## File Tab

All the files in a tree structure. Can press one to open in Patch Tab.

Bottom bar with search and dropdown:
[All files]
[Conflicts only]
[Modified only]

Problems
    - How to handle DLTX patches?
        - Don't treat them like regular file. In VFS scan, instead of adding them to their literal path, add them to the file they modify in a dltx field.
    - How to handle files that patch deletes?
        - File just has DELETE in it.
        - App reads it but user can also understand these files in their own IDE.

## Patch Tab

The titlebar is of course the file path and then the patch manager. (Status, Patch / Unpatch)
Then another table/list. The patch is not treated like a normal mod in this list.

Priority | Mod | Hash | Patch Hash | Status | Notes | Action |

You can edit Patch Hash & Notes, and press the Action button
Action is a custom command defined in the configuration that provides %patch% and %addition%. E.g. code --diff %patch% %addition%.
Action will be disabled if the hashes match or the patch hash is IGNORE. Right click > Open Anyway can bypass this.
Status is just a ✅ (hash match), ⏪ (hash mismatch), or ❌ (no patch)

Problems
    - Action is quite limited and clunky.
        ? IDE Plugin
        ? open link
        ? Integrated editor
        ? Right click stuff instead
    - DLTX
        - Seperate table, marked special.
        - Additional action to patch / unpatch with DELETE.
    - Patch / Unpatch functionality
        - Patch
            - Makes you select the base you want to use, OR lets you use DELETE.
        - Unpatch
            - Confirm remove.
    - Images
    - Direct patch from existing file

## Safety & Folders

If you copy over your whole instance to ANYWHERE else, it should work EXACTLY the same.
Required:
    configs - The order data, patch data, and settings.
    patch - The patch files.
    mods - Contains mod folders. Technically you can have mods other places but it's not cool.
Optional:
    archives - The original archive for the mod. Not nessacary but makes reinstallation easier.
    instance - You could have your instance anywhere. But it should probably be here.
Not recommended:
    VFS - The current VFS-modified instance. Symlinks will *probably* break if their origins are transferred, so let the software just generate a new one. (Also not cross-platform)