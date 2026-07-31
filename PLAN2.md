# Patch Plan V2

## Diffs & Overrides

Files are replaced with a diff-formatted file, which can be merged together to create a final file.

Overrides are done per-final-file. Each file that is modified in the patch gets a folder. The folder contains a JSON file to manage the data, and all the nesscesary diff-formatted files.

Each source can be linked to a diff-formatted file that will be used, by hash of the original file and the mod it comes from.

Supported diff formats:
|Format|Support|Type|
|------|-------|----|
|.ltx|YES|TBD|
|.xml|YES|TBD|

Of course, few file types can be safely diffed. 

## Final File

Depending on the type of file, different methods must be used.

### Diff Merge & Patch

For file types that use diff-formats, sources are merged together to create the final file used in the VFS. Sources can be selected / deselected at will and reordered for priority. In this format, INTERNAL conflicts can arise when two diff-files files override the same part. The user can create ONE _modpack_ diff-file to resolve this, which is always highest priority, and is very special in that it does not have a direct source, but is instead linked to the conflicting sections themselves. (NOT the diff-files, the sections.)  

### Manual Merge

For file types that do not support diff-formats, a user must manually select a file or provide a custom merged one TODO linked?. This option is also available for diff-formats but strongly discouraged.

## Editor / Updating

An editor is of course needed for manual diff-conversion and creation of modpack files. The most important part is not just the text editing, but also the metadata and the underlying changes that drive it.

### Diff-format source editing.

Upon a source changing hash, the previously linked diff-file will be marked as out-of-date. The editor must show side-by-side the new copy's changes and the diff-format. The user can edit the diff-format to match changes and re-link to the new source. Notes:

- Diff generation is not a particularly hard task, this can most often be done without the editor, with just one click. (See DLTXify)
    
- The source that is linked to ANY diff-formatted file must be saved seperately, for update comparison.

This also applies to initial creation, instead using the base game's file as the reference for the diff.

### Modpack diff-format editing

To create a modpack diff-file, the user can see the full final file on one side, and the newly created diff-file on the other. The final file has an overlay over all the spots that have been changed, and internal conflicts show as multiple conflicting implementations for the same section. The user can then create a replacement for that section which is linked to the sections it is a resolution from.

When linked sections change, the modpack diff-file is now marked out-of-date. A similar editor to above is shown, but also with a diff from the original linked section and the new section that replaced it. Works the same as before. 

### Other file editing.

Certain formats (most notably .script) do have rhyme and reason to their workings, but can't exactly be differential. This should probably also have a method of creation that can compare differences TODO what.

## JSON

The JSON is per-final-file.

In diff-format supporting formats, it contains 
- A diff-file list, each entry containing:
    - Path to diff-file
    - Path to source file backup
    - Source file hash
    - Source file mod name
    - Source file mod path (both so if one changes, it will update to never lose track)
    - Reference ID (unchanging)
    - Enabled status
- A modpack diff-file entry, containing:
    - Path to diff-file
    - List of linked diff-file hashes and reference IDs
    - List of linked section backups and host reference IDs
In all formats, it contains:

- Path to override file OR selection 

Everything is optional, as all nessacary non-patch data is pulled directly from the VFS and mod data, so a empty patch is actually empty.