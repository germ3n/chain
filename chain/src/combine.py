# combine_rust_files.py
#
# A simple Python script to combine all Rust source files (.rs) in a directory
# and its subdirectories into a single file, wrapping each file's content
# in a Markdown code block.
#
# USAGE:
# 1. Place this script in the root of your Rust source directory (e.g., 'src').
# 2. Run the script from your terminal: `python combine_rust_files.py`
# 3. A new file named 'combined.rs' will be created in the same directory,
#    containing the content of all your .rs files.

import os

def combine_rust_files():
    """
    Finds all .rs files in the current directory and subdirectories,
    and combines them into a single output file with header comments and
    Markdown code blocks.
    """
    # The name of the file that will contain the combined source code.
    output_filename = "combined.rs"
    
    # The directory where the script is running.
    source_directory = '.'
    
    # A list to hold the relative paths of all found .rs files.
    rust_files = []

    print("Searching for Rust source files...")

    # Walk through the source directory and its subdirectories.
    # os.walk('.') starts the search from the current directory.
    for root, _, files in os.walk(source_directory):
        for filename in files:
            # We only care about files that end with the .rs extension.
            # We also exclude this script and the output file itself to avoid recursion.
            if filename.endswith(".rs") and filename != output_filename and filename != os.path.basename(__file__):
                # Create a relative path from the source_directory.
                # This makes the paths in the comments cleaner (e.g., "lib.rs" or "module/mod.rs").
                relative_path = os.path.join(root, filename)
                rust_files.append(relative_path)

    # Sorting the list of files ensures a consistent and predictable
    # order in the output file every time the script is run.
    rust_files.sort()

    if not rust_files:
        print("No .rs files found in this directory.")
        return

    print(f"Found {len(rust_files)} files. Combining them into '{output_filename}'...")

    try:
        # Open the output file in 'write' mode ('w').
        # This will create the file if it doesn't exist, or overwrite it if it does.
        # Using 'with' ensures the file is automatically closed even if errors occur.
        with open(output_filename, 'w', encoding='utf-8') as outfile:
            # Loop through each filepath we found.
            for idx, filepath in enumerate(rust_files):
                print(f"  -> Adding {filepath}")
                try:
                    # Open the current Rust source file in 'read' mode ('r').
                    with open(filepath, 'r', encoding='utf-8') as infile:
                        # Get the content of the file.
                        content = infile.read()

                        # Format the header comment as requested.
                        # os.path.normpath cleans up the path (e.g., ./module/file.rs -> module/file.rs)
                        header = f"//===FILE: {os.path.normpath(filepath)}//\n"
                        
                        # Write the header and the content to the output file.
                        outfile.write(header)
                        # Start the markdown code block, specifying 'rust' for syntax highlighting.
                        outfile.write("```rust\n")
                        outfile.write(content)
                        # Close the markdown code block.
                        outfile.write("\n```")

                        # Add two newlines after each file's content for readability,
                        # but not after the very last file.
                        if idx < len(rust_files) - 1:
                            outfile.write("\n\n")

                except Exception as e:
                    print(f"    [!] Error reading file {filepath}: {e}")
    
    except IOError as e:
        print(f"[!] Error: Could not write to output file '{output_filename}'. Reason: {e}")
        return

    print(f"\n✅ Success! All files have been combined into '{output_filename}'.")


if __name__ == "__main__":
    combine_rust_files()
