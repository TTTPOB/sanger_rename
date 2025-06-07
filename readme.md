# Sanger Rename

A TUI tool for renaming Sanger sequencing files from different vendors into a standardized format.

## Supported Vendors

- **Sangon** - e.g., `0001_31225060307072_(TXPCR)_[SP1].ab1` → `250601.TXPCR.SP1.ab1`
- **Ruibio** - e.g., `K528-1.C1.34781340.B08.ab1` → `251206.K528-1.C1.ab1`
- **Genewiz** - e.g., `TL1-T25_A01.ab1` → `250601.TL1.T25.ab1`
  
  ⚠️ **Note**: Genewiz support is not well tested since they don't have a properly defined separator for each part of the filename.

## Usage

```bash
# Single file
sanger_rename file.ab1

# Multiple files
sanger_rename *.ab1
```

The TUI will guide you through:
1. Vendor selection
2. Template/primer name editing
3. Date selection
4. Confirmation before renaming

## Windows Send To Context Menu

For easy access, add this to your Windows "Send To" menu:

1. Press `Win + R`
2. Type `shell:sendto` and hit Enter
3. Create a shortcut to `sanger_rename.exe` in that folder

Now you can right-click on selected multiple `.ab1`/`.seq` files and select "Send to" → "sanger_rename".