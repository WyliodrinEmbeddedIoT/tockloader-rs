# tbf-parser

This document is a list of changes made to the original crate.

## Changes:
- `get_application_flags`
    - Return flags of the application
    - DELTA: Originally did not exist
- `get_protected_trailer_size`
    - Get the size of the protected trailer. This only returns the trailer size, WITHOUT the header. The app cannot write to this region.
    - DELTA: Originally did not exist
- `get_protected_region_size`
    - Get the size in bytes of the protected region from the beginning of the process binary (start of the TBF header). The returned size includes the TBF Header. Only valid if this is an app.
    - DELTA: Originally named `get_protected_size`, renamed to remove ambiguity.
- `get_tbf_version` 
    - Return the version of the Tock Binary Format
    - DELTA: Originally did not exist