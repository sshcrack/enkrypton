declare const __TAURI_INVOKE__: any;

export function invokeWindowTauri(window: string, cmd: string, payload: any) {
    return __TAURI_INVOKE__("tauri", {
        __tauriModule: 'Window',
        message: {
            cmd: 'manage',
            data: {
                label: window,
                cmd: {
                    type: cmd,
                    payload: payload
                }
            }
        }
    }) as Promise<void>
}