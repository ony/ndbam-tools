## Main goals

* Update metadata for installed packages to take/withdraw ownership for
  new/updated files or registering additional metadata.
* Reduced responsibility to allow more freedom. For example resolver might be
  quiet complicated and just produce install/upgrade/remove scenario.
* Suitable for manual recovery on a partially corrupted system. I.e. it should
  be as simple as unpacking archive in filesystem, but still preserve
  consistency of metadata.
* Managed collisions with manual control of safety during install/remove:
    - *no-conflicts*: only install packages with no conflicting files.
    - *allow-identical*: keep files of same content and avoid removing
      referenced files.
    - *yield*: create new files with slightly different names and intended to
      be merged into existing files by other tools.
    - *clobber*: simply overwrite old files and remove only own content.
* Support many-to-many package replacing.
* Maintain proper metadata describing installed packages.

## Metadata format

It is expected to be used in Exherbo and thus it should be exndbam/vdb
compatible format. It is expected that other tools may want associate with
packages additional information and provide more advanced indexes.
*More to define*

## Install sources

Image of root folder with additional information like name and optional version.
*More to define*
