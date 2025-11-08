#!/bin/bash
# Parse Claude JSON output to show only what the model is doing

while IFS= read -r line; do
    # Check if this is an assistant message
    if echo "$line" | jq -e '.type == "assistant"' >/dev/null 2>&1; then
        # Extract text content (if any)
        text_content=$(echo "$line" | jq -r '.message.content[]? | select(.type == "text") | .text' 2>/dev/null)
        if [ -n "$text_content" ]; then
            echo "💬 $text_content"
            echo
        fi

        # Extract tool uses
        tool_info=$(echo "$line" | jq -r '.message.content[]? | select(.type == "tool_use") |
            if .name == "TodoWrite" then
                "📝 Managing todo list:\n" +
                (.input.todos[]? | "   [\(.status)] \(.content)" | gsub("in_progress"; "▶️ in_progress") | gsub("completed"; "✅ completed") | gsub("pending"; "⏸️  pending"))
            elif .name == "Bash" then
                "🔧 \(.input.description // "Running bash command")\n   $ \(.input.command)"
            elif .name == "Write" then
                "✍️  Writing file: \(.input.file_path)"
            elif .name == "Edit" then
                "✏️  Editing file: \(.input.file_path)"
            elif .name == "Read" then
                "📖 Reading file: \(.input.file_path)"
            elif .name == "Grep" then
                "🔍 Searching for: \(.input.pattern)"
            else
                "🔧 Using tool: \(.name)" + (if .input.description then " - \(.input.description)" else "" end)
            end' 2>/dev/null)

        if [ -n "$tool_info" ]; then
            echo -e "$tool_info"
            echo
        fi
    fi
done
