package app.localnative.android;

public class RustBridge {
    static {
        System.loadLibrary("localnative_core");
    }
    private static native String localnativeRun(final String pattern);
    public static String run(String input) {
        return localnativeRun(input);
    }
}
