#!/usr/bin/env node

const { Binary } = require('binary-install');
const os = require('os');
const { join } = require('path');

const version = require('./package.json').version;
const name = 'rustyhook';
const binName = 'rh';

function getPlatform() {
  const type = os.type();
  const arch = os.arch();

  if (type === 'Windows_NT') {
    return {
      platform: 'windows',
      arch: arch === 'x64' ? 'x86_64' : arch,
      extension: '.exe',
      binary: 'rh.exe'
    };
  }

  if (type === 'Linux') {
    return {
      platform: 'linux',
      arch: arch === 'x64' ? 'x86_64' : arch,
      extension: '',
      binary: 'rh'
    };
  }

  if (type === 'Darwin') {
    return {
      platform: 'darwin',
      arch: arch === 'x64' ? 'x86_64' : arch,
      extension: '',
      binary: 'rh'
    };
  }

  throw new Error(`Unsupported platform: ${type} ${arch}`);
}

function getBinary() {
  const { platform, arch, extension, binary } = getPlatform();
  const url = `https://github.com/your-org/rustyhook/releases/download/v${version}/rustyhook-v${version}-${arch}-${platform}${extension === '.exe' ? '.zip' : '.tar.gz'}`;
  
  return new Binary(url, { name: binName, installDirectory: join(__dirname, 'bin') });
}

const binary = getBinary();
binary.install();