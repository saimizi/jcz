# Shell Completions for jcz

This directory contains shell completion scripts for jcz.

## Fish Shell

### Installation

To install the fish completion, copy the completion file to your fish completions directory:

```bash
# System-wide installation (requires sudo)
sudo cp completions/jcz.fish /usr/share/fish/vendor_completions.d/

# User installation
mkdir -p ~/.config/fish/completions
cp completions/jcz.fish ~/.config/fish/completions/
```

### Testing

To test the completion without installing it permanently, you can source it in your current fish session:

```bash
source completions/jcz.fish
```

### Usage

Once installed, the fish shell will automatically provide completions for jcz commands. Try typing:

```bash
jcz -c <TAB>        # Shows available compression commands
jcz -t <TAB>        # Shows timestamp options
jcz -d <TAB>        # Shows compressed files for decompression
```

### Features

The fish completion provides:
- All command-line options with descriptions
- Compression format suggestions (gzip, bzip2, xz, zip, tar, tgz, tbz2, txz)
- Compression level values (1-9)
- Timestamp option values (0-3)
- Context-aware completions:
  - Encryption options only in compression mode
  - Decryption options only in decompression mode
  - File filtering based on mode (compressed files for -d, all files otherwise)
- Directory completion for -C/--move-to option
- File path completion for encryption/decryption keys

## Bash Shell

### Installation

To install the bash completion, copy the completion file to your bash completions directory:

```bash
# System-wide installation (requires sudo)
sudo cp completions/jcz.bash /usr/share/bash-completion/completions/jcz

# User installation (for bash-completion 2.x)
mkdir -p ~/.local/share/bash-completion/completions
cp completions/jcz.bash ~/.local/share/bash-completion/completions/jcz

# Alternative user installation
mkdir -p ~/.bash_completion.d
cp completions/jcz.bash ~/.bash_completion.d/jcz.bash
echo 'source ~/.bash_completion.d/jcz.bash' >> ~/.bashrc
```

### Testing

To test the completion without installing it permanently, you can source it in your current bash session:

```bash
source completions/jcz.bash
```

### Usage

Once installed, bash will automatically provide completions for jcz commands. Try typing:

```bash
jcz -c <TAB>        # Shows available compression commands
jcz -l <TAB>        # Shows compression levels (1-9)
jcz -t <TAB>        # Shows timestamp options
jcz -d <TAB>        # Shows compressed files for decompression
```

### Features

The bash completion provides:
- All command-line options
- Compression format suggestions (gzip, bzip2, xz, zip, tar, tgz, tbz2, txz)
- Compression level values (1-9)
- Timestamp option values (0-3)
- Context-aware completions:
  - Encryption options only in compression mode
  - Decryption options only in decompression mode
  - File filtering based on mode (compressed files for -d, all files otherwise)
- Directory-only completion for -C/--move-to option
- File path completion for encryption/decryption keys

## Future Completions

Additional shell completions (zsh, powershell) can be added to this directory in the future.
