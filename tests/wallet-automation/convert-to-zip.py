import os
import sys
import zipfile
from glob import glob
from datetime import datetime

# Function to find the most recent .crx file in a given directory
def find_most_recent_crx(directory):
    crx_files = glob(os.path.join(directory, '*.crx'))
    if not crx_files:
        return None
    latest_file = max(crx_files, key=os.path.getmtime)
    return latest_file

# Get the directory of the current script (__dirname equivalent)
script_dir = os.path.dirname(os.path.abspath(__file__))

# Resolve the 'extensions' path relative to the script's directory
extensions_dir = os.path.join(script_dir, 'typhon/extensions')

def crx_to_zip(crx_file_path, zip_file_path=None):
    if not zip_file_path:
        # If no zip file path is provided, use the 'extensions' directory
        base_name = os.path.basename(os.path.splitext(crx_file_path)[0])
        zip_file_path = os.path.join(extensions_dir, base_name + '.zip')
    
    with open(crx_file_path, 'rb') as crx_file:
        # Skip the CRX file's header (first 16 bytes)
        crx_file.read(16)
        # The actual zip content starts after the header
        zip_content = crx_file.read()
        
        # Save the zip content to a new zip file
        with open(zip_file_path, 'wb') as zip_file:
            zip_file.write(zip_content)
    
    print(f"Converted CRX to ZIP: {zip_file_path}")
    return zip_file_path

def unzip_file(zip_path, extract_to=None):
    if extract_to is None:
        extract_to = extensions_dir
    if not os.path.exists(extract_to):
        os.makedirs(extract_to)
    with zipfile.ZipFile(zip_path, 'r') as zip_ref:
        zip_ref.extractall(extract_to)
        print(f"Extracted all files in {zip_path} to {extract_to}")

if __name__ == "__main__":
    download_dir = extensions_dir  # Or any other directory where .crx files are downloaded

    # Ensure the 'extensions' directory exists
    if not os.path.exists(download_dir):
        os.makedirs(download_dir)

    crx_path = find_most_recent_crx(download_dir)
    if crx_path is None:
        print(f"No .crx files found in {download_dir}.")
        sys.exit(1)

    zip_path = crx_to_zip(crx_path)  # Convert CRX to ZIP
    unzip_file(zip_path)  # Unzip the converted file
