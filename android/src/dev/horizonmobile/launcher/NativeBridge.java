package dev.horizonmobile.launcher;

import android.view.Surface;

/**
 * JNI Native Bridge linking Android Java UI to the compiled Rust Horizon 26.3
 * client library and launcher core (`libhorizon_launcher_android.so`).
 */
public class NativeBridge {
    static {
        try {
            System.loadLibrary("horizon_launcher_android");
        } catch (UnsatisfiedLinkError e) {
            e.printStackTrace();
        }
    }

    /**
     * Initializes and executes the native Horizon 26.3 Rust client loop.
     */
    public static native int launchGame(
        String gameDir,
        String renderer,
        float renderScale,
        int memoryLimitMib,
        Surface surface
    );

    /**
     * Sends touchpad relative delta motions (dx, dy) to game camera or mouse cursor.
     */
    public static native void sendTouchpadDelta(float dx, float dy);

    /**
     * Sends virtual button action events (PRI, SEC, Jump, Sneak, Hotbar).
     */
    public static native void sendButtonAction(int actionId, boolean isDown);

    /**
     * Returns hardware diagnostics JSON probed from the Linux kernel.
     */
    public static native String getDiagnosticsJson();

    /**
     * Recommends optimal low-end preset based on detected RAM.
     */
    public static native String recommendPresetJson(long totalRamMib, int cores);
}
