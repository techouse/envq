Output docs/assets/envq-demo.gif

Require cargo

Set Shell "bash"
Set Width 1840
Set Height 1240
Set Padding 48
Set FontSize 30
Set Framerate 24
Set TypingSpeed 1ms

Hide
Type "PROMPT_COMMAND=; PS1=$'\\[\\033[32m\\]~\\[\\033[0m\\] \\[\\033[36m\\]$\\[\\033[0m\\] '"
Enter
Type "cargo build --release --quiet"
Enter
Sleep 2s
Type "export PATH=$PWD/target/release:$PATH"
Enter
Type "demo_dir=$(mktemp -d)"
Enter
Type "cd $demo_dir"
Enter
Type "cat > .env <<'EOF'"
Enter
Type "DATABASE_URL=postgres://local"
Enter
Type "API_TOKEN=old-token"
Enter
Type "FEATURE_FLAG=false"
Enter
Type "# comments stay in place"
Enter
Type "EOF"
Enter
Type "printf 'from-secret-file' > token.txt"
Enter
Type "demo_comment_index=0"
Enter
Type "demo_comment_prompt() {"
Enter
Type "  case $demo_comment_index in"
Enter
Type "    0) printf '\033[32m~\033[0m \033[36m$\033[0m \033[38;5;245m%s\033[0m\n' '# start with a plain .env file' ;;"
Enter
Type "    1) printf '\033[32m~\033[0m \033[36m$\033[0m \033[38;5;245m%s\033[0m\n' '# check for a key using only exit status' ;;"
Enter
Type "    2) printf '\033[32m~\033[0m \033[36m$\033[0m \033[38;5;245m%s\033[0m\n' '# preview a value piped from another file' ;;"
Enter
Type "    3) printf '\033[32m~\033[0m \033[36m$\033[0m \033[38;5;245m%s\033[0m\n' '# write that stdin value in place' ;;"
Enter
Type "    4) printf '\033[32m~\033[0m \033[36m$\033[0m \033[38;5;245m%s\033[0m\n' '# read the updated value back exactly' ;;"
Enter
Type "    5) printf '\033[32m~\033[0m \033[36m$\033[0m \033[38;5;245m%s\033[0m\n' '# clear a value without removing its key' ;;"
Enter
Type "    6) printf '\033[32m~\033[0m \033[36m$\033[0m \033[38;5;245m%s\033[0m\n' '# keep the key and write an empty value' ;;"
Enter
Type "    7) printf '\033[32m~\033[0m \033[36m$\033[0m \033[38;5;245m%s\033[0m\n' '# emit JSON for scripts' ;;"
Enter
Type "    *) return ;;"
Enter
Type "  esac"
Enter
Type "  demo_comment_index=$((demo_comment_index + 1))"
Enter
Type "}"
Enter
Type "clear; PROMPT_COMMAND=demo_comment_prompt"
Enter
Sleep 500ms
Set TypingSpeed 55ms
Show
Sleep 800ms

Type "cat .env"
Enter
Sleep 1s

Type "envq has FEATURE_FLAG .env && echo FEATURE_FLAG exists"
Enter
Sleep 1s

Type "cat token.txt | envq set API_TOKEN - .env --diff"
Enter
Sleep 2s

Type "cat token.txt | envq set API_TOKEN - .env"
Enter
Sleep 600ms

Type "envq get API_TOKEN .env; echo"
Enter
Sleep 1s

Type "envq clear FEATURE_FLAG .env --diff"
Enter
Sleep 2s

Type "envq clear FEATURE_FLAG .env"
Enter
Sleep 600ms

Type "envq list .env --json"
Enter
Sleep 2s
