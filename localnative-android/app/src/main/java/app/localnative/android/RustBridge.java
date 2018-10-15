package app.localnative.android;

public class RustBridge {
    private static native String localnativeRun(final String pattern);

    public String run(String input) {
        return localnativeRun(input);
    }
}
