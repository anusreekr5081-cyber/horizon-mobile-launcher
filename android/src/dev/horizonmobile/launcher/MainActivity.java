package dev.horizonmobile.launcher;

import android.app.Activity;
import android.content.Intent;
import android.os.Bundle;
import android.view.View;
import android.widget.ArrayAdapter;
import android.widget.Button;
import android.widget.EditText;
import android.widget.SeekBar;
import android.widget.Spinner;
import android.widget.TextView;
import java.io.File;

/**
 * Main PojavLauncher / Mojo Launcher styled dashboard for Horizon 26.3.
 */
public class MainActivity extends Activity {

    private EditText editPlayerName;
    private Spinner spinnerRenderer;
    private Spinner spinnerControls;
    private TextView txtRamLabel;
    private SeekBar seekRam;
    private TextView txtScaleLabel;
    private SeekBar seekScale;
    private Button btnLaunch;

    private int selectedRamMib = 1024;
    private float selectedScale = 0.75f;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);

        editPlayerName = findViewById(R.id.edit_player_name);
        spinnerRenderer = findViewById(R.id.spinner_renderer);
        spinnerControls = findViewById(R.id.spinner_controls);
        txtRamLabel = findViewById(R.id.txt_ram_label);
        seekRam = findViewById(R.id.seek_ram);
        txtScaleLabel = findViewById(R.id.txt_scale_label);
        seekScale = findViewById(R.id.seek_scale);
        btnLaunch = findViewById(R.id.btn_launch);

        // Renderer options
        String[] renderers = new String[] { "Auto (Recommended)", "gl4es (OpenGL 2.1)", "Zink (Vulkan)", "VirGL (Hardware)", "ANGLE" };
        ArrayAdapter<String> rendererAdapter = new ArrayAdapter<>(this, android.R.layout.simple_spinner_dropdown_item, renderers);
        spinnerRenderer.setAdapter(rendererAdapter);

        // Controls options
        String[] controls = new String[] { "Pojav Virtual Touchpad (Recommended)", "Compact Low-End Layout", "Default Phone Layout" };
        ArrayAdapter<String> controlsAdapter = new ArrayAdapter<>(this, android.R.layout.simple_spinner_dropdown_item, controls);
        spinnerControls.setAdapter(controlsAdapter);

        // RAM Slider (256 MiB to 4096 MiB)
        seekRam.setMax(3840);
        seekRam.setProgress(768); // 256 + 768 = 1024 MiB
        seekRam.setOnSeekBarChangeListener(new SeekBar.OnSeekBarChangeListener() {
            @Override
            public void onProgressChanged(SeekBar seekBar, int progress, boolean fromUser) {
                selectedRamMib = 256 + progress;
                txtRamLabel.setText("ALLOCATED RAM: " + selectedRamMib + " MiB");
            }
            @Override public void onStartTrackingTouch(SeekBar seekBar) {}
            @Override public void onStopTrackingTouch(SeekBar seekBar) {}
        });

        // Scale Slider (50% to 100%)
        seekScale.setMax(50);
        seekScale.setProgress(25); // 50 + 25 = 75%
        seekScale.setOnSeekBarChangeListener(new SeekBar.OnSeekBarChangeListener() {
            @Override
            public void onProgressChanged(SeekBar seekBar, int progress, boolean fromUser) {
                int pct = 50 + progress;
                selectedScale = pct / 100.0f;
                txtScaleLabel.setText("RENDER SCALE: " + pct + "%");
            }
            @Override public void onStartTrackingTouch(SeekBar seekBar) {}
            @Override public void onStopTrackingTouch(SeekBar seekBar) {}
        });

        // Quick Preset Buttons
        findViewById(R.id.btn_preset_ultralow).setOnClickListener(new View.OnClickListener() {
            @Override public void onClick(View v) {
                seekRam.setProgress(256); // 512 MB
                seekScale.setProgress(0);  // 50%
            }
        });

        findViewById(R.id.btn_preset_balanced).setOnClickListener(new View.OnClickListener() {
            @Override public void onClick(View v) {
                seekRam.setProgress(768); // 1024 MB
                seekScale.setProgress(25); // 75%
            }
        });

        findViewById(R.id.btn_preset_high).setOnClickListener(new View.OnClickListener() {
            @Override public void onClick(View v) {
                seekRam.setProgress(1792); // 2048 MB
                seekScale.setProgress(50);  // 100%
            }
        });

        findViewById(R.id.btn_preset_extreme).setOnClickListener(new View.OnClickListener() {
            @Override public void onClick(View v) {
                seekRam.setProgress(3840); // 4096 MB
                seekScale.setProgress(50);  // 100%
            }
        });

        // PLAY Button
        btnLaunch.setOnClickListener(new View.OnClickListener() {
            @Override
            public void onClick(View v) {
                launchGame();
            }
        });
    }

    private void launchGame() {
        String playerName = editPlayerName.getText().toString().trim();
        if (playerName.isEmpty()) {
            playerName = "Player";
        }

        String rawRenderer = spinnerRenderer.getSelectedItem().toString();
        String renderer = "auto";
        if (rawRenderer.contains("gl4es")) renderer = "gl4es";
        else if (rawRenderer.contains("Zink")) renderer = "zink";
        else if (rawRenderer.contains("VirGL")) renderer = "virgl";
        else if (rawRenderer.contains("ANGLE")) renderer = "angle";

        File instanceDir = new File(getFilesDir(), "instances/" + playerName);
        instanceDir.mkdirs();

        Intent intent = new Intent(this, GameActivity.class);
        intent.putExtra("game_dir", instanceDir.getAbsolutePath());
        intent.putExtra("renderer", renderer);
        intent.putExtra("ram_mib", selectedRamMib);
        intent.putExtra("render_scale", selectedScale);
        startActivity(intent);
    }
}
