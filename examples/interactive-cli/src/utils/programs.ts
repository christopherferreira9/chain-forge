import { existsSync, readdirSync, readFileSync } from 'fs';
import { join, dirname } from 'path';
import { ProgramInfo } from '../types';

const PROGRAMS_DIR = join(__dirname, '..', '..', 'programs');

export function discoverPrograms(): ProgramInfo[] {
  const programs: ProgramInfo[] = [];

  if (!existsSync(PROGRAMS_DIR)) {
    return programs;
  }

  const entries = readdirSync(PROGRAMS_DIR, { withFileTypes: true });

  for (const entry of entries) {
    if (entry.isDirectory()) {
      const cargoTomlPath = join(PROGRAMS_DIR, entry.name, 'Cargo.toml');
      if (existsSync(cargoTomlPath)) {
        const cargoToml = readFileSync(cargoTomlPath, 'utf-8');
        const nameMatch = cargoToml.match(/name\s*=\s*"([^"]+)"/);
        const name = nameMatch ? nameMatch[1] : entry.name;

        programs.push({
          name,
          path: join(PROGRAMS_DIR, entry.name),
          cargoTomlPath,
        });
      }
    }
  }

  return programs;
}

export function getProgramSoPath(program: ProgramInfo): string {
  return join(program.path, 'target', 'deploy', `${program.name}.so`);
}

export function isProgramBuilt(program: ProgramInfo): boolean {
  return existsSync(getProgramSoPath(program));
}
