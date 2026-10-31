package dev.horizonmobile.launcher;

import android.content.Context;
import android.graphics.Canvas;
import android.graphics.Color;
import android.graphics.Paint;
import android.graphics.RectF;
import android.util.AttributeSet;
import android.view.MotionEvent;
import android.view.View;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

/**
 * High-performance, low-latency on-screen gamepad & virtual touchpad overlay
 * modeled directly on PojavLauncher and Mojo Launcher touch controls.
 */
public class TouchpadOverlayView extends View {

    public static final int ACTION_MOVE_FORWARD = 0;
    public static final int ACTION_MOVE_BACKWARD = 1;
    public static final int ACTION_MOVE_LEFT = 2;
    public static final int ACTION_MOVE_RIGHT = 3;
    public static final int ACTION_JUMP = 4;
    public static final int ACTION_SNEAK = 5;
    public static final int ACTION_SPRINT = 6;
    public static final int ACTION_ATTACK_PRI = 7;
    public static final int ACTION_USE_SEC = 8;
    public static final int ACTION_MID_PICK = 9;
    public static final int ACTION_INVENTORY = 10;
    public static final int ACTION_PAUSE_ESC = 11;
    public static final int ACTION_CHAT = 12;
    public static final int ACTION_TAB = 13;
    public static final int ACTION_F3 = 14;
    public static final int ACTION_F5 = 15;
    public static final int ACTION_HOTBAR_BASE = 20; // 21 to 29 for slots 1-9

    public static class VirtualBtn {
        public String id;
        public String label;
        public int actionId;
        public float rx, ry, rw, rh; // Normalized coords (0..1)
        public boolean isToggle;
        public boolean isToggled;
        public boolean isPressed;
        public RectF screenBounds = new RectF();

        public VirtualBtn(String id, String label, int actionId, float rx, float ry, float rw, float rh, boolean isToggle) {
            this.id = id;
            this.label = label;
            this.actionId = actionId;
            this.rx = rx;
            this.ry = ry;
            this.rw = rw;
            this.rh = rh;
            this.isToggle = isToggle;
        }

        public void updateBounds(int width, int height) {
            screenBounds.set(rx * width, ry * height, (rx + rw) * width, (ry + rh) * height);
        }

        public boolean contains(float x, float y) {
            return screenBounds.contains(x, y);
        }
    }

    private final List<VirtualBtn> buttons = new ArrayList<>();
    private final Map<Integer, VirtualBtn> pointerBtnMap = new HashMap<>();

    // Touchpad state
    private final RectF touchpadBounds = new RectF();
    private int touchpadPointerId = -1;
    private float lastTouchpadX = 0f;
    private float lastTouchpadY = 0f;
    private boolean mouseMode = false;
    private float cursorX = 0.5f;
    private float cursorY = 0.5f;
    private float sensitivity = 1.25f;

    // Paints
    private final Paint btnPaint = new Paint(Paint.ANTI_ALIAS_FLAG);
    private final Paint btnPressedPaint = new Paint(Paint.ANTI_ALIAS_FLAG);
    private final Paint btnTextPaint = new Paint(Paint.ANTI_ALIAS_FLAG);
    private final Paint touchpadPaint = new Paint(Paint.ANTI_ALIAS_FLAG);
    private final Paint cursorPaint = new Paint(Paint.ANTI_ALIAS_FLAG);

    public TouchpadOverlayView(Context context) {
        super(context);
        init();
    }

    public TouchpadOverlayView(Context context, AttributeSet attrs) {
        super(context, attrs);
        init();
    }

    private void init() {
        btnPaint.setColor(Color.argb(170, 30, 35, 45));
        btnPaint.setStyle(Paint.Style.FILL);

        btnPressedPaint.setColor(Color.argb(220, 46, 204, 113));
        btnPressedPaint.setStyle(Paint.Style.FILL);

        btnTextPaint.setColor(Color.WHITE);
        btnTextPaint.setTextSize(28f);
        btnTextPaint.setTextAlign(Paint.Align.CENTER);
        btnTextPaint.setFakeBoldText(true);

        touchpadPaint.setColor(Color.argb(35, 52, 152, 219));
        touchpadPaint.setStyle(Paint.Style.FILL);

        cursorPaint.setColor(Color.argb(240, 241, 196, 15));
        cursorPaint.setStyle(Paint.Style.FILL);

        setupPojavLayout();
    }

    private void setupPojavLayout() {
        buttons.clear();
        // D-Pad / Movement
        buttons.add(new VirtualBtn("w", "▲", ACTION_MOVE_FORWARD, 0.06f, 0.62f, 0.07f, 0.12f, false));
        buttons.add(new VirtualBtn("s", "▼", ACTION_MOVE_BACKWARD, 0.06f, 0.80f, 0.07f, 0.12f, false));
        buttons.add(new VirtualBtn("a", "◀", ACTION_MOVE_LEFT, 0.01f, 0.71f, 0.06f, 0.12f, false));
        buttons.add(new VirtualBtn("d", "▶", ACTION_MOVE_RIGHT, 0.12f, 0.71f, 0.06f, 0.12f, false));
        buttons.add(new VirtualBtn("center_jump", "◆", ACTION_JUMP, 0.065f, 0.73f, 0.06f, 0.08f, false));

        // Pojav Action buttons (Right)
        buttons.add(new VirtualBtn("pri", "PRI", ACTION_ATTACK_PRI, 0.86f, 0.52f, 0.08f, 0.14f, false));
        buttons.add(new VirtualBtn("sec", "SEC", ACTION_USE_SEC, 0.76f, 0.52f, 0.08f, 0.14f, false));
        buttons.add(new VirtualBtn("mid", "MID", ACTION_MID_PICK, 0.81f, 0.40f, 0.07f, 0.11f, false));
        buttons.add(new VirtualBtn("mouse", "MOUSE", -1, 0.91f, 0.40f, 0.07f, 0.11f, true));
        buttons.add(new VirtualBtn("jump", "JUMP", ACTION_JUMP, 0.86f, 0.70f, 0.09f, 0.16f, false));
        buttons.add(new VirtualBtn("snk", "SNK", ACTION_SNEAK, 0.76f, 0.70f, 0.08f, 0.12f, true));

        // Top bar
        buttons.add(new VirtualBtn("esc", "ESC", ACTION_PAUSE_ESC, 0.01f, 0.02f, 0.06f, 0.07f, false));
        buttons.add(new VirtualBtn("tab", "TAB", ACTION_TAB, 0.08f, 0.02f, 0.06f, 0.07f, false));
        buttons.add(new VirtualBtn("f3", "F3", ACTION_F3, 0.38f, 0.02f, 0.05f, 0.07f, false));
        buttons.add(new VirtualBtn("f5", "F5", ACTION_F5, 0.44f, 0.02f, 0.05f, 0.07f, false));
        buttons.add(new VirtualBtn("chat", "CHAT", ACTION_CHAT, 0.50f, 0.02f, 0.07f, 0.07f, false));
        buttons.add(new VirtualBtn("inv", "INV", ACTION_INVENTORY, 0.46f, 0.78f, 0.08f, 0.08f, false));

        // 1-9 Hotbar slots (Bottom)
        for (int i = 1; i <= 9; i++) {
            float sx = 0.23f + ((i - 1) * 0.06f);
            buttons.add(new VirtualBtn("slot_" + i, String.valueOf(i), ACTION_HOTBAR_BASE + i, sx, 0.89f, 0.055f, 0.09f, false));
        }
    }

    @Override
    protected void onSizeChanged(int w, int h, int oldw, int oldh) {
        super.onSizeChanged(w, h, oldw, oldh);
        for (VirtualBtn btn : buttons) {
            btn.updateBounds(w, h);
        }
        // Touchpad spans 0.25 to 0.75 horizontally, 0.15 to 0.75 vertically
        touchpadBounds.set(0.24f * w, 0.14f * h, 0.74f * w, 0.76f * h);
    }

    @Override
    protected void onDraw(Canvas canvas) {
        super.onDraw(canvas);

        // Draw touchpad boundary guide
        canvas.drawRoundRect(touchpadBounds, 16f, 16f, touchpadPaint);

        // Draw buttons
        for (VirtualBtn btn : buttons) {
            Paint p = (btn.isPressed || btn.isToggled) ? btnPressedPaint : btnPaint;
            canvas.drawRoundRect(btn.screenBounds, 12f, 12f, p);

            float cx = btn.screenBounds.centerX();
            float cy = btn.screenBounds.centerY() - ((btnTextPaint.descent() + btnTextPaint.ascent()) / 2f);
            canvas.drawText(btn.label, cx, cy, btnTextPaint);
        }

        // Draw virtual mouse cursor if mouseMode is on
        if (mouseMode) {
            float mx = cursorX * getWidth();
            float my = cursorY * getHeight();
            canvas.drawCircle(mx, my, 10f, cursorPaint);
        }
    }

    @Override
    public boolean onTouchEvent(MotionEvent event) {
        int action = event.getActionMasked();
        int index = event.getActionIndex();
        int pointerId = event.getPointerId(index);

        switch (action) {
            case MotionEvent.ACTION_DOWN:
            case MotionEvent.ACTION_POINTER_DOWN: {
                float px = event.getX(index);
                float py = event.getY(index);
                handlePointerDown(pointerId, px, py);
                break;
            }
            case MotionEvent.ACTION_MOVE: {
                for (int i = 0; i < event.getPointerCount(); i++) {
                    int pId = event.getPointerId(i);
                    float px = event.getX(i);
                    float py = event.getY(i);
                    handlePointerMove(pId, px, py);
                }
                break;
            }
            case MotionEvent.ACTION_POINTER_UP:
            case MotionEvent.ACTION_UP: {
                handlePointerUp(pointerId);
                break;
            }
            case MotionEvent.ACTION_CANCEL: {
                pointerBtnMap.clear();
                for (VirtualBtn btn : buttons) {
                    if (!btn.isToggle) btn.isPressed = false;
                }
                touchpadPointerId = -1;
                invalidate();
                break;
            }
        }
        return true;
    }

    private void handlePointerDown(int pointerId, float x, float y) {
        // Check buttons
        for (VirtualBtn btn : buttons) {
            if (btn.contains(x, y)) {
                pointerBtnMap.put(pointerId, btn);
                if (btn.isToggle) {
                    btn.isToggled = !btn.isToggled;
                    if ("mouse".equals(btn.id)) {
                        mouseMode = btn.isToggled;
                    }
                } else {
                    btn.isPressed = true;
                }
                if (btn.actionId >= 0) {
                    NativeBridge.sendButtonAction(btn.actionId, true);
                }
                invalidate();
                return;
            }
        }

        // If no button was hit, check touchpad area
        if (touchpadBounds.contains(x, y)) {
            touchpadPointerId = pointerId;
            lastTouchpadX = x;
            lastTouchpadY = y;
        }
    }

    private void handlePointerMove(int pointerId, float x, float y) {
        if (pointerId == touchpadPointerId) {
            float dx = (x - lastTouchpadX) * sensitivity;
            float dy = (y - lastTouchpadY) * sensitivity;

            // Send relative delta to game engine
            NativeBridge.sendTouchpadDelta(dx, dy);

            // Update virtual cursor position
            cursorX = Math.max(0f, Math.min(1f, cursorX + (dx / getWidth())));
            cursorY = Math.max(0f, Math.min(1f, cursorY + (dy / getHeight())));

            lastTouchpadX = x;
            lastTouchpadY = y;
            invalidate();
        }
    }

    private void handlePointerUp(int pointerId) {
        VirtualBtn btn = pointerBtnMap.remove(pointerId);
        if (btn != null) {
            if (!btn.isToggle) {
                btn.isPressed = false;
                if (btn.actionId >= 0) {
                    NativeBridge.sendButtonAction(btn.actionId, false);
                }
            }
            invalidate();
        }

        if (pointerId == touchpadPointerId) {
            touchpadPointerId = -1;
        }
    }
}
