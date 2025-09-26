# Install

## _reshuffle_apps(apps, preserve_order=False)

This internal helper function determines the **exact physical arrangement of applications in flash memory** before installing them on a Tock board.  
It enforces constraints from **TBF headers**, **board-specific flash/RAM layout rules**, and **MPU alignment requirements** so that the kernel can correctly discover and run applications.

---

### Purpose

Tock applications are stored in flash in **TBF (Tock Binary Format)** containers.  
Each TBF header specifies:
- Whether the app is **fixed-address** (must be loaded at a specific flash/RAM address) or **position-independent** (can be loaded anywhere).
- The app’s **flash size** and **RAM usage**.
- MPU alignment requirements.

> **Note on fixed addresses and sizes:**  
> A fixed flash address refers to where the **TBF header itself** (and thus the whole app) must be placed in flash.  
> The size used for layout calculations **includes** the TBF header plus the app’s binary contents.  
> There is no footer — the kernel uses the total size from the header to locate the next app.

The kernel discovers apps by scanning flash **sequentially** from the **start-of-apps** address.  
Because of this, the **order and alignment** of apps in flash is critical.  
_reshuffle_apps() ensures:
- Correct **sorting** of apps.
- Proper **MPU-compliant alignment**.
- Insertion of **padding** when needed.
- Prevention of **flash and RAM overlaps**.

---

### Determining the Correct Order

The function first determines which type of applications it is dealing with.

#### **1. Fixed-address apps**
- Compiled for **specific flash (and sometimes RAM) addresses**.
- The addresses are encoded in the TBF header and cannot be changed without recompilation.
- _reshuffle_apps() will:
  1. Sort them in **increasing flash address order**.
  2. Verify **no overlapping flash ranges**.
  3. If RAM addresses are fixed, verify **no overlapping RAM ranges**.
  4. Discard any app starting **before the start-of-apps address** or outside flash limits.
  5. Insert **padding apps** into gaps so that kernel scanning continues correctly.

**Why sorting by address is mandatory**  
The kernel loads apps like this:
address = start_of_apps
while valid_TBF_header_at(address):
    load_app()
    address += app_size

#### 2. Position-independent apps

- Can be placed **anywhere** in flash because they use relative addressing.
- _reshuffle_apps() decides order based on:
  - **preserve_order=True** → Keep apps exactly in the order they are given.
  - **preserve_order=False**:
    - "size_descending" → Sort largest to smallest to reduce fragmentation.
    - None → Any order is acceptable.
- After ordering, applies alignment rules and inserts padding when needed.

---

#### 3. Mixed fixed and position-independent

- **Not supported**.  
  Mixing fixed and movable apps is complex and currently unimplemented.
- If both types are detected, _reshuffle_apps() raises a TockLoaderException.

---

### Alignment and Padding

In this context, **alignment** means placing each app in flash at a starting address that matches certain hardware rules.  
These rules come from the MPU (Memory Protection Unit) or the flash controller, and they define which addresses are considered valid starting points for an app.

For example:
- Some boards require apps to start at a multiple of their own size.
- Others require starting addresses to be aligned to a power-of-two boundary (e.g., 4 KB).
- Some flash chips can only write or erase in page-sized chunks.

If an app would start at an address that doesn’t meet these rules, `_reshuffle_apps()` inserts a **padding app** — an empty region filled with `0xFF`.  
This moves the start of the next real app forward until it’s at a valid aligned address.

Padding apps have no executable code. They simply reserve space so that:
- The kernel can correctly find the next app when scanning flash.
- Hardware alignment requirements are satisfied.


### Cleaning Up Flash

After writing, _reshuffle_apps():

- Erases flash immediately after the last installed app.
- Ensures the kernel detects the correct **end-of-applications**.
- Prevents stray flash data from being mistaken for a valid app.