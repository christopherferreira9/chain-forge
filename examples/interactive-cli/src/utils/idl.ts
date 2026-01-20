import { existsSync, readFileSync } from 'fs';
import { join, dirname, basename } from 'path';
import { ProgramIDL, ProgramInfo } from '../types';

/**
 * Get the expected IDL file path for a program.
 * IDL files should be named <program_name>.idl.json and placed in the program directory.
 */
export function getIDLPath(program: ProgramInfo): string {
  return join(program.path, `${program.name}.idl.json`);
}

/**
 * Check if an IDL file exists for a program.
 */
export function hasIDL(program: ProgramInfo): boolean {
  return existsSync(getIDLPath(program));
}

/**
 * Load an IDL file for a program.
 * Returns null if the IDL file doesn't exist or is invalid.
 */
export function loadIDL(idlPath: string): ProgramIDL | null {
  if (!existsSync(idlPath)) {
    return null;
  }

  try {
    const content = readFileSync(idlPath, 'utf-8');
    const idl = JSON.parse(content) as ProgramIDL;

    // Basic validation
    if (!idl.name || !idl.version || !Array.isArray(idl.instructions)) {
      return null;
    }

    return idl;
  } catch {
    return null;
  }
}

/**
 * Load an IDL file for a program by its info.
 */
export function loadIDLForProgram(program: ProgramInfo): ProgramIDL | null {
  const idlPath = getIDLPath(program);
  return loadIDL(idlPath);
}
