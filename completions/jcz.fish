# Fish completion for jcz - Just Compress Zip
# A unified compression utility

# Helper functions to check conditions
function __fish_jcz_using_decompress
    __fish_contains_opt -s d decompress
end

function __fish_jcz_not_using_decompress
    not __fish_jcz_using_decompress
end

# Main options
complete -c jcz -s d -l decompress -d "Decompress mode"
complete -c jcz -s f -l force -d "Force overwrite without prompting"

# Compression command
complete -c jcz -s c -l command -d "Compression command" -x
complete -c jcz -s c -l command -a "gzip" -d "GZIP compression (.gz)" -x
complete -c jcz -s c -l command -a "bzip2" -d "BZIP2 compression (.bz2)" -x
complete -c jcz -s c -l command -a "xz" -d "XZ compression (.xz)" -x
complete -c jcz -s c -l command -a "zip" -d "ZIP compression (.zip)" -x
complete -c jcz -s c -l command -a "tar" -d "TAR archive (.tar)" -x
complete -c jcz -s c -l command -a "tgz" -d "TAR + GZIP (.tar.gz)" -x
complete -c jcz -s c -l command -a "tbz2" -d "TAR + BZIP2 (.tar.bz2)" -x
complete -c jcz -s c -l command -a "txz" -d "TAR + XZ (.tar.xz)" -x

# Compression level
complete -c jcz -s l -l level -d "Compression level (1-9)" -x
complete -c jcz -s l -l level -a "1 2 3 4 5 6 7 8 9" -x

# Move output to directory
complete -c jcz -s C -l move-to -d "Move output to specified directory" -r -F

# Collection options
complete -c jcz -s a -l collect -d "Collect files into archive (with parent directory)" -x
complete -c jcz -s A -l collect-flat -d "Collect files into archive (flat, without parent directory)" -x

# Timestamp option
complete -c jcz -s t -l timestamp -d "Timestamp option" -x
complete -c jcz -s t -l timestamp -a "0" -d "No timestamp" -x
complete -c jcz -s t -l timestamp -a "1" -d "Date timestamp" -x
complete -c jcz -s t -l timestamp -a "2" -d "Datetime timestamp" -x
complete -c jcz -s t -l timestamp -a "3" -d "Nanoseconds timestamp" -x

# Encryption options (only in compression mode)
complete -c jcz -s e -l encrypt-password -d "Enable password-based encryption" -n __fish_jcz_not_using_decompress
complete -c jcz -l encrypt-key -d "RSA public key file for encryption" -r -F -n __fish_jcz_not_using_decompress

# Decryption options (only in decompression mode)
complete -c jcz -l decrypt-key -d "RSA private key file for decryption" -r -F -n __fish_jcz_using_decompress
complete -c jcz -l remove-encrypted -d "Remove encrypted file after successful decryption" -n __fish_jcz_using_decompress

# File completion for input files
# In decompress mode, suggest compressed files
complete -c jcz -n __fish_jcz_using_decompress -a "(__fish_complete_suffix .gz .bz2 .xz .zip .tar .tgz .tbz2 .txz .jcze)" -d "Compressed file"

# In compress mode, suggest all files
complete -c jcz -n __fish_jcz_not_using_decompress -F -d "File or directory to compress"
