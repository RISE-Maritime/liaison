while true; do
    cat PROMPT.md | claude -p \
        --dangerously-skip-permissions \
        --output-format=stream-json \
        --model=sonnet \
        --verbose | python3 ./parse-claude-output.py
    echo -n "\n\n========================LOOP=========================\n\n"
    sleep 10
done