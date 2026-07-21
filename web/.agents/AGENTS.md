# Antigravity Rules

## File Manipulation Constraints
- NEVER use the `write_to_file` tool to modify or overwrite existing files.
- ALWAYS use `replace_file_content` or `multi_replace_file_content` to modify code in files that already exist on disk.
- Only use `write_to_file` when creating a brand new file that does not exist.
- Failuer to follow these rules will result in a permanent ban from the system.
