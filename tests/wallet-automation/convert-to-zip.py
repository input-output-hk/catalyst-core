import os
import sys
import zipfile

def crx_to_zip(crx_file_path, zip_file_path=None):
    if not zip_file_path:
        # If no zip file path is provided, use the same location and name as the crx file
        zip_file_path = os.path.splitext(crx_file_path)[0] + '.zip'
    
    with open(crx_file_path, 'rb') as crx_file:
        # Read the CRX file's header (first 16 bytes)
        crx_header = crx_file.read(16)
        
        # The actual zip content starts after the header, so we can skip the header
        zip_content = crx_file.read()
        
        # Save the zip content to a new zip file
        with open(zip_file_path, 'wb') as zip_file:
            zip_file.write(zip_content)
    
    print(f"converted CRX to ZIP: {zip_file_path}")
    return zip_file_path

def unzip_file(zip_path, extract_to=None):
    if extract_to is None:
        extract_to = os.path.dirname(zip_path)
    
    if not os.path.exists(extract_to):
        os.makedirs(extract_to)

    with zipfile.ZipFile(zip_path, 'r') as zip_ref:
        zip_ref.extractall(extract_to)
        print(f"Extracted all files in {zip_path} to {extract_to}")

# Combine the operations
if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python script.py <path_to_crx_file>")
        sys.exit(1)

    crx_path = sys.argv[1]
    zip_path = crx_to_zip(crx_path)  # Convert CRX to ZIP
    unzip_file(zip_path)  # Unzip the converted file
