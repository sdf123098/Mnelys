import { beforeEach, describe, expect, it, vi } from 'vitest';

import { CoreCommandError, core } from './client';

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}));

describe('the Tauri client', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it('passes the command name and empty argument object to invoke', async () => {
    invokeMock.mockResolvedValue({ version: '0.1.0', contractVersion: 1, profile: 'debug' });

    await expect(core.version()).resolves.toEqual({
      version: '0.1.0',
      contractVersion: 1,
      profile: 'debug',
    });
    expect(invokeMock).toHaveBeenCalledWith('core_version', {});
  });

  it('turns a structured core error into a CoreCommandError with its code intact', async () => {
    invokeMock.mockRejectedValue({ code: 'core/not-implemented', message: 'not yet' });

    await expect(core.version()).rejects.toBeInstanceOf(CoreCommandError);
    await expect(core.version()).rejects.toMatchObject({
      code: 'core/not-implemented',
      message: 'not yet',
    });
  });

  it('treats an unstructured rejection as an internal core error', async () => {
    invokeMock.mockRejectedValue('the backend exploded');

    await expect(core.runtimeInfo()).rejects.toMatchObject({
      code: 'core/internal',
      message: 'the backend exploded',
    });
  });

  it('falls back to a generic message when the rejection carries nothing usable', async () => {
    invokeMock.mockRejectedValue({ code: 'not-a-real-code', message: 'ignored' });

    await expect(core.runtimeInfo()).rejects.toMatchObject({ code: 'core/internal' });
  });
});
