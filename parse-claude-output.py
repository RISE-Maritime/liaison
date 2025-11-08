#!/usr/bin/env python3
"""Parse Claude JSON output to show only what the model is doing."""

import json
import sys


def format_todo_status(status):
    """Format todo status with emoji."""
    emoji_map = {
        "in_progress": "▶️",
        "completed": "✅",
        "pending": "⏸️"
    }
    return emoji_map.get(status, "⚪")


def format_tool_use(tool):
    """Format a tool use for display."""
    tool_name = tool.get("name", "Unknown")
    tool_input = tool.get("input", {})

    if tool_name == "TodoWrite":
        lines = ["📝 Managing todo list:"]
        for todo in tool_input.get("todos", []):
            status = todo.get("status", "unknown")
            content = todo.get("content", "")
            emoji = format_todo_status(status)
            lines.append(f"   [{emoji} {status}] {content}")
        return "\n".join(lines)

    elif tool_name == "Bash":
        desc = tool_input.get("description", "Running bash command")
        cmd = tool_input.get("command", "")
        return f"🔧 {desc}\n   $ {cmd}"

    elif tool_name == "Write":
        path = tool_input.get("file_path", "")
        return f"✍️  Writing file: {path}"

    elif tool_name == "Edit":
        path = tool_input.get("file_path", "")
        return f"✏️  Editing file: {path}"

    elif tool_name == "Read":
        path = tool_input.get("file_path", "")
        return f"📖 Reading file: {path}"

    elif tool_name == "Grep":
        pattern = tool_input.get("pattern", "")
        return f"🔍 Searching for: {pattern}"

    elif tool_name == "Glob":
        pattern = tool_input.get("pattern", "")
        return f"🔎 Finding files: {pattern}"

    elif tool_name == "Task":
        desc = tool_input.get("description", "")
        subagent = tool_input.get("subagent_type", "")
        return f"🤖 Launching agent: {subagent} - {desc}"

    else:
        desc = tool_input.get("description", "")
        if desc:
            return f"🔧 {tool_name}: {desc}"
        return f"🔧 {tool_name}"


def main():
    """Process each line of JSON input."""
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue

        try:
            data = json.loads(line)

            # Only process assistant messages
            if data.get("type") != "assistant":
                continue

            message = data.get("message", {})
            content = message.get("content", [])

            # Process each content block
            for block in content:
                if block.get("type") == "text":
                    text = block.get("text", "").strip()
                    if text:
                        print(f"💬 {text}")
                        print()

                elif block.get("type") == "tool_use":
                    tool_output = format_tool_use(block)
                    if tool_output:
                        print(tool_output)
                        print()

        except json.JSONDecodeError:
            # Skip malformed JSON
            continue
        except Exception as e:
            # Skip lines that cause errors
            continue


if __name__ == "__main__":
    main()
