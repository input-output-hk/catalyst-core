# arguments: jormungandr version, target triple, target cpu

import sys
import platform
import hashlib
import shutil
import os


def sha256sum(path):
    h = hashlib.sha256()
    with open(path, 'rb') as f:
        data = f.read()
        h.update(data)
    return h.hexdigest()


version = sys.argv[1]
target = sys.argv[2]
target_cpu = sys.argv[3]

archive_basename = 'vit-testing-{}-{}-{}'.format(version, target, target_cpu)

root_dir = './target/{}/release'.format(target)

if platform.system() == 'Windows':
    iapyx_cli_name = 'iapyx-cli.exe'
    iapyx_load_name = 'iapyx-load.exe'
    iapyx_proxy_name = 'iapyx-proxy.exe'
    iapyx_qr_name = 'iapyx-qr.exe'
    vitup_name = 'vitup.exe'
    vitup_cli_name = 'vitup-cli.exe'
    snapshot_service_name = 'snapshot-trigger-service.exe'
    snapshot_cli_name = 'snapshot-cli.exe'
    registration_service_name = 'registration-service.exe'
else:
    iapyx_cli_name = 'iapyx-cli'
    iapyx_load_name = 'iapyx-load'
    iapyx_proxy_name = 'iapyx-proxy'
    iapyx_qr_name = 'iapyx-qr'
    vitup_name = 'vitup'
    vitup_cli_name = 'vitup-cli'
    snapshot_service_name = 'snapshot-trigger-service'
    snapshot_cli_name = 'snapshot-cli'
    registration_service_name = 'registration-service'

iapyx_cli_path = os.path.join(root_dir, iapyx_cli_name)
iapyx_load_path = os.path.join(root_dir, iapyx_load_name)
iapyx_proxy_path = os.path.join(root_dir, iapyx_proxy_name)
iapyx_qr_path = os.path.join(root_dir, iapyx_qr_name)
vitup_path = os.path.join(root_dir, vitup_name)
vitup_cli_path = os.path.join(root_dir, vitup_cli_name)
snapshot_service_path= os.path.join(root_dir, snapshot_service_name)
snapshot_cli_path = os.path.join(root_dir, snapshot_cli_name)
registration_service_path = os.path.join(root_dir, registration_service_name)

iapyx_cli_checksum = sha256sum(iapyx_cli_path)
iapyx_load_checksum = sha256sum(iapyx_load_path)
iapyx_proxy_checksum = sha256sum(iapyx_proxy_path)
iapyx_qr_checksum = sha256sum(iapyx_qr_path)
vitup_checksum = sha256sum(vitup_path)
vitup_cli_checksum = sha256sum(vitup_cli_path)
snapshot_service_checksum = sha256sum(snapshot_service_path)
snapshot_cli_checksum = sha256sum(snapshot_cli_path)
registration_service_checksum = sha256sum(registration_service_path)

# build archive
if platform.system() == 'Windows':
    import zipfile
    content_type = 'application/zip'
    archive_name = '{}.zip'.format(archive_basename)
    with zipfile.ZipFile(archive_name, mode='x') as archive:
        archive.write(iapyx_cli_path, arcname=iapyx_cli_name)
        archive.write(iapyx_load_path, arcname=iapyx_load_name)
        archive.write(iapyx_proxy_path, arcname=iapyx_proxy_name)
        archive.write(iapyx_qr_path, arcname=iapyx_qr_name)
        archive.write(vitup_path, arcname=vitup_name)
        archive.write(vitup_cli_path, arcname=vitup_cli_name)
        archive.write(snapshot_service_path, arcname=snapshot_service_name)
        archive.write(snapshot_cli_path, arcname=snapshot_cli_name)
        archive.write(registration_service_path, arcname=registration_service_name)
        
else:
    import tarfile
    content_type = 'application/gzip'
    archive_name = '{}.tar.gz'.format(archive_basename)
    with tarfile.open(archive_name, 'x:gz') as archive:
        archive.write(iapyx_cli_path, arcname=iapyx_cli_name)
        archive.write(iapyx_load_path, arcname=iapyx_load_name)
        archive.write(iapyx_proxy_path, arcname=iapyx_proxy_name)
        archive.write(iapyx_qr_path, arcname=iapyx_qr_name)
        archive.write(vitup_path, arcname=vitup_name)
        archive.write(vitup_cli_path, arcname=vitup_cli_name)
        archive.write(snapshot_service_path, arcname=snapshot_service_name)
        archive.write(snapshot_cli_path, arcname=snapshot_cli_name)
        archive.write(registration_service_path, arcname=registration_service_name)

# verify archive
shutil.unpack_archive(archive_name, './unpack-test')
iapyx_cli1_checksum = sha256sum(os.path.join('./unpack-test', iapyx_cli_name))
iapyx_load1_checksum = sha256sum(os.path.join('./unpack-test', iapyx_load_name))
iapyx_proxy1_checksum = sha256sum(os.path.join('./unpack-test', iapyx_proxy_name))
iapyx_qr1_checksum = sha256sum(os.path.join('./unpack-test', iapyx_qr_name))
vitup1_checksum = sha256sum(os.path.join('./unpack-test', vitup_name))
vitup_cli1_checksum = sha256sum(os.path.join('./unpack-test', vitup_cli_name))
snapshot_service1_checksum = sha256sum(os.path.join('./unpack-test', snapshot_service_name))
snapshot_cli1_checksum = sha256sum(os.path.join('./unpack-test', snapshot_cli_name))
registration_service1_checksum = sha256sum(os.path.join('./unpack-test', registration_service_name))


shutil.rmtree('./unpack-test')
if iapyx_cli1_checksum != iapyx_cli_checksum:
    print('iapyx cli checksum mismarch: before {} != after {}'.format(
        iapyx_cli_checksum, iapyx_cli1_checksum))
    exit(1)
if iapyx_load1_checksum != iapyx_load_checksum:
    print('iapyx load checksum mismarch: before {} != after {}'.format(
        iapyx_load_checksum, iapyx_load1_checksum))
    exit(1)
if iapyx_proxy1_checksum != iapyx_proxy_checksum:
    print('iapyx proxy checksum mismarch: before {} != after {}'.format(
        iapyx_proxy_checksum, iapyx_proxy1_checksum))
    exit(1)
if iapyx_qr1_checksum != iapyx_qr_checksum:
    print('iapyx qr checksum mismarch: before {} != after {}'.format(
        iapyx_qr_checksum, iapyx_qr1_checksum))
    exit(1)
if vitup1_checksum != vitup_checksum:
    print('vitup checksum mismarch: before {} != after {}'.format(
        vitup_checksum, vitup1_checksum))
    exit(1)
if snapshot_service1_checksum != snapshot_service_checksum:
    print('snapshot service checksum mismarch: before {} != after {}'.format(
        snapshot_service_checksum, snapshot_service1_checksum))
    exit(1)
    if snapshot_cli1_checksum != snapshot_cli_checksum:
    print('snapshot cli checksum mismarch: before {} != after {}'.format(
        snapshot_cli_checksum, snapshot_cli1_checksum))
    exit(1)
if registration_service1_checksum != registration_service_checksum:
    print('registration service checksum mismarch: before {} != after {}'.format(
        registration_service_checksum, registration_service1_checksum))
    exit(1)
if vitup_cli1_checksum != vitup_cli_checksum:
    print('vitup checksum mismarch: before {} != after {}'.format(
        vitup_cli_checksum, vitup_cli1_checksum))
    exit(1)

# save archive checksum
archive_checksum = sha256sum(archive_name)
checksum_filename = '{}.sha256'.format(archive_name)
with open(checksum_filename, 'x') as f:
    f.write(archive_checksum)

# set GitHub Action step outputs
print('::set-output name=release-archive::{}'.format(archive_name))
print('::set-output name=release-content-type::{}'.format(content_type))
