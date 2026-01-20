import chalk from 'chalk';

export function truncateAddress(address: string, chars: number = 4): string {
  if (address.length <= chars * 2 + 3) {
    return address;
  }
  return `${address.slice(0, chars)}...${address.slice(-chars)}`;
}

export function formatBalance(balance: number): string {
  return balance.toFixed(2);
}

export function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export function padRight(str: string, length: number): string {
  return str.padEnd(length);
}

export function padLeft(str: string, length: number): string {
  return str.padStart(length);
}

export function success(message: string): string {
  return chalk.green(message);
}

export function error(message: string): string {
  return chalk.red(message);
}

export function warning(message: string): string {
  return chalk.yellow(message);
}

export function info(message: string): string {
  return chalk.cyan(message);
}

export function dim(message: string): string {
  return chalk.dim(message);
}

export function bold(message: string): string {
  return chalk.bold(message);
}
