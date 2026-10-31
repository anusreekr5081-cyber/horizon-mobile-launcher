package dev.horizonmobile.launcher;

import android.app.Activity;
import android.os.Bundle;
import android.view.Surface;
import android.view.SurfaceHolder;
import android.view.SurfaceView;
import android.view.Window;
import android.view.WindowManager;

/**
 * In-game runtime host running Horizon 26.3 with full virtual gamepad & touchpad overlay.
 */
public class GameActivity extends Activity implements SurfaceHolder.Callback {

    private SurfaceView surfaceView;
    private String gameDir;
    private String renderer;
    private float renderScale;
    private int memoryLimitMib;
    private Thread gameThread;
    private boolean isRunning = false;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        requestWindowFeature(Window.FEATURE_NO_TITLE);
        getWindow().setFlags(
            WindowManager.LayoutParams.FLAG_FULLSCREEN | WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON,
            WindowManager.LayoutParams.FLAG_FULLSCREEN | WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON
        );

        setContentView(R.layout.activity_game);

        gameDir = getIntent().getStringExtra("game_dir");
        if (gameDir == null) {
            gameDir = getFilesDir().getAbsolutePath() + "/instances/default";
        }
        renderer = getIntent().getStringExtra("renderer");
        if (renderer == null) {
            renderer = "auto";
        }
        renderScale = getIntent().getFloatExtra("render_scale", 0.75f);
        memoryLimitMib = getIntent().getIntExtra("ram_mib", 1024);

        surfaceView = findViewById(R.id.surface_game);
        surfaceView.getHolder().addCallback(this);
    }

    @Override
    public void surfaceCreated(SurfaceHolder holder) {
        final Surface surface = holder.getSurface();
        isRunning = true;
        gameThread = new Thread(new Runnable() {
            @Override
            public void run() {
                NativeBridge.launchGame(
                    gameDir,
                    renderer,
                    renderScale,
                    memoryLimitMib,
                    surface
                );
            }
        }, "HorizonGameThread");
        gameThread.start();
    }

    @Override
    public void surfaceChanged(SurfaceHolder holder, int format, int width, int height) {
    }

    @Override
    public void surfaceDestroyed(SurfaceHolder holder) {
        isRunning = false;
        if (gameThread != null) {
            try {
                gameThread.join(1000);
            } catch (InterruptedException ignored) {}
        }
    }
}
