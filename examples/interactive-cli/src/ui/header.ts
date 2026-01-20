import chalk from 'chalk';
import Table from 'cli-table3';
import { getActiveNode } from '../state';
import { truncateAddress, formatBalance, dim, bold } from './formatters';

const BOX_WIDTH = 65;

function horizontalLine(char: string, left: string, right: string): string {
  return left + char.repeat(BOX_WIDTH - 2) + right;
}

function paddedLine(content: string): string {
  const visibleLength = content.replace(/\x1b\[[0-9;]*m/g, '').length;
  // Total: ║ (1) + left pad (2) + content + padding + right pad (2) + ║ (1) = 65
  // So: content + padding = 65 - 6 = 59
  const padding = 59 - visibleLength;
  return `║  ${content}${' '.repeat(Math.max(0, padding))}  ║`;
}

export function renderHeader(): void {
  console.clear();

  const node = getActiveNode();

  // Top border
  console.log(chalk.cyan(horizontalLine('═', '╔', '╗')));

  // Title
  console.log(chalk.cyan(paddedLine(bold('Chain Forge Interactive CLI'))));

  if (node) {
    // Node info
    const nodeInfo = `Nodes: ${chalk.green('1')} | Port: ${chalk.yellow(node.port.toString())} | Accounts: ${chalk.yellow(node.accounts.length.toString())}`;
    console.log(chalk.cyan(paddedLine(nodeInfo)));

    // Separator
    console.log(chalk.cyan(horizontalLine('═', '╠', '╣')));

    // Accounts header
    const header = `${dim('#')}  │ ${dim('Address')}                                    │ ${dim('Balance')}`;
    console.log(chalk.cyan(paddedLine(header)));

    // Account rows
    for (let i = 0; i < Math.min(node.accounts.length, 5); i++) {
      const account = node.accounts[i];
      const index = i.toString().padStart(2, ' ');
      const address = truncateAddress(account.publicKey, 8);
      const balance = formatBalance(account.balance).padStart(9, ' ');
      const row = `${index} │ ${address.padEnd(42)} │ ${balance}`;
      console.log(chalk.cyan(paddedLine(row)));
    }

    if (node.accounts.length > 5) {
      console.log(chalk.cyan(paddedLine(dim(`   ... and ${node.accounts.length - 5} more accounts`))));
    }

    // Programs section
    if (node.deployedPrograms.length > 0) {
      console.log(chalk.cyan(horizontalLine('─', '╟', '╢')));
      console.log(chalk.cyan(paddedLine(dim('Deployed Programs:'))));
      for (const program of node.deployedPrograms) {
        const deployerShort = truncateAddress(program.deployerAddress, 4);
        const programInfo = `  ${program.name} → ${truncateAddress(program.programId, 8)} ${dim(`by ${deployerShort}`)}`;
        console.log(chalk.cyan(paddedLine(programInfo)));
      }
    }

    // Program accounts section
    if (node.programAccounts.length > 0) {
      console.log(chalk.cyan(horizontalLine('─', '╟', '╢')));
      console.log(chalk.cyan(paddedLine(dim(`Program Accounts (${node.programAccounts.length}):`))));
      for (const pa of node.programAccounts.slice(0, 3)) {
        // Find the program name for this account
        const program = node.deployedPrograms.find((p) => p.programId === pa.programId);
        const programName = program ? program.name : truncateAddress(pa.programId, 4);
        const accountInfo = `  ${truncateAddress(pa.address, 8)} ${dim(`owned by ${programName}`)}`;
        console.log(chalk.cyan(paddedLine(accountInfo)));
      }
      if (node.programAccounts.length > 3) {
        console.log(chalk.cyan(paddedLine(dim(`   ... and ${node.programAccounts.length - 3} more`))));
      }
    }
  } else {
    console.log(chalk.cyan(paddedLine(dim('No active node'))));
  }

  // Bottom border
  console.log(chalk.cyan(horizontalLine('═', '╚', '╝')));
  console.log();
}
