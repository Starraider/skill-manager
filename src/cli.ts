#!/usr/bin/env node
import { Command } from 'commander';

const program = new Command();
program.name('skill-manager').description('Install local skill bundles into AI tools').version('0.1.0');
program.option('--config <path>', 'Path to config.yaml');
program.action(async () => {
  const { run } = await import('./prompts.js');
  await run(program.opts<{ config?: string }>().config);
});
await program.parseAsync(process.argv);
