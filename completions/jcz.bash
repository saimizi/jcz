# Bash completion for jcz - Just Compress Zip
# A unified compression utility

_jcz_completion() {
    local cur prev words cword
    _init_completion || return

    local opts="-d --decompress -f --force -c --command -l --level -C --move-to -a --collect -A --collect-flat -t --timestamp -e --encrypt-password --encrypt-key --decrypt-key --remove-encrypted -h --help -V --version"
    local commands="gzip bzip2 xz zip tar tgz tbz2 txz"
    local levels="1 2 3 4 5 6 7 8 9"
    local timestamps="0 1 2 3"

    # Check if we're in decompress mode
    local decompress_mode=0
    for ((i=1; i < ${#words[@]}; i++)); do
        if [[ "${words[i]}" == "-d" || "${words[i]}" == "--decompress" ]]; then
            decompress_mode=1
            break
        fi
    done

    case "${prev}" in
        -c|--command)
            COMPREPLY=( $(compgen -W "${commands}" -- "${cur}") )
            return 0
            ;;
        -l|--level)
            COMPREPLY=( $(compgen -W "${levels}" -- "${cur}") )
            return 0
            ;;
        -t|--timestamp)
            case "${cur}" in
                0) COMPREPLY=("0 # No timestamp") ;;
                1) COMPREPLY=("1 # Date timestamp") ;;
                2) COMPREPLY=("2 # Datetime timestamp") ;;
                3) COMPREPLY=("3 # Nanoseconds timestamp") ;;
                *) COMPREPLY=( $(compgen -W "${timestamps}" -- "${cur}") ) ;;
            esac
            return 0
            ;;
        -C|--move-to)
            # Complete directories only
            _filedir -d
            return 0
            ;;
        -a|--collect|-A|--collect-flat)
            # Archive name, no completion
            return 0
            ;;
        --encrypt-key|--decrypt-key)
            # Complete files (likely .pem files)
            _filedir
            return 0
            ;;
    esac

    # Handle options
    if [[ "${cur}" == -* ]]; then
        if [[ ${decompress_mode} -eq 1 ]]; then
            # In decompress mode, exclude compression-only options
            local decompress_opts="-d --decompress -f --force -C --move-to --decrypt-key --remove-encrypted -h --help -V --version"
            COMPREPLY=( $(compgen -W "${decompress_opts}" -- "${cur}") )
        else
            # In compress mode, exclude decompression-only options
            local compress_opts="-d --decompress -f --force -c --command -l --level -C --move-to -a --collect -A --collect-flat -t --timestamp -e --encrypt-password --encrypt-key -h --help -V --version"
            COMPREPLY=( $(compgen -W "${compress_opts}" -- "${cur}") )
        fi
        return 0
    fi

    # File completion
    if [[ ${decompress_mode} -eq 1 ]]; then
        # In decompress mode, suggest compressed files
        local compressed_exts="@(gz|bz2|xz|zip|tar|tgz|tbz2|txz|jcze)"
        _filedir "${compressed_exts}"
    else
        # In compress mode, suggest all files and directories
        _filedir
    fi

    return 0
}

complete -F _jcz_completion jcz
