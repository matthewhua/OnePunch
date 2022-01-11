package io.matthew.poidemo.util;

import org.apache.poi.util.TempFile;

import java.io.File;

public class TempFileUtils {
    private TempFileUtils() {
    }

    @SuppressWarnings("java:S106")
    public static void checkTempFiles() {
        String tmpDir = System.getProperty(TempFile.JAVA_IO_TMPDIR) + "/poifiles";
        final File tempDir = new File(tmpDir);
        if(tempDir.exists()){
            final String[] tempFiles = tempDir.list();
            if (tempFiles != null && tempFiles.length > 0) {
                System.out.println("found files in poi temp dir " + tempDir.getAbsolutePath());
                for (String filename : tempFiles) {
                    System.out.println("file: " + filename);
                }
            }
        }else {
            System.out.println("unable to find poi temp dir");
        }
    }

}
