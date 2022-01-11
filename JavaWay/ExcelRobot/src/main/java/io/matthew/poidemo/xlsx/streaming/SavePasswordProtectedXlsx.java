package io.matthew.poidemo.xlsx.streaming;


import io.matthew.poidemo.util.TempFileUtils;
import org.apache.poi.util.IOUtils;
import org.apache.poi.openxml4j.exceptions.InvalidFormatException;
import org.apache.poi.openxml4j.opc.OPCPackage;
import org.apache.poi.poifs.crypt.EncryptionInfo;
import org.apache.poi.poifs.crypt.EncryptionMode;
import org.apache.poi.poifs.crypt.Encryptor;
import org.apache.poi.poifs.crypt.temp.EncryptedTempData;
import org.apache.poi.poifs.crypt.temp.SXSSFWorkbookWithCustomZipEntrySource;
import org.apache.poi.poifs.filesystem.POIFSFileSystem;
import org.apache.poi.xssf.streaming.SXSSFCell;
import org.apache.poi.xssf.streaming.SXSSFRow;
import org.apache.poi.xssf.streaming.SXSSFSheet;

import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.security.GeneralSecurityException;

@SuppressWarnings({"java:S106","java:S4823","java:S1192"})
public class SavePasswordProtectedXlsx {

    private SavePasswordProtectedXlsx() {}

    public static void main(String[] args) throws Exception {

        if(args.length != 2) {
            throw new IllegalArgumentException("Expected 2 params: filename and password");
        }
        TempFileUtils.checkTempFiles();
        String filename = args[0];
        String password = args[1];
        SXSSFWorkbookWithCustomZipEntrySource wb = new SXSSFWorkbookWithCustomZipEntrySource();

        try {
            for(int i = 0; i < 10; i++) {
                 SXSSFSheet sheet = wb.createSheet("Sheet" + i);
                for (int r = 0; r < 1000; r++) {
                     SXSSFRow row = sheet.createRow(r);
                    for (int c = 0; c < 1000; c++) {
                        SXSSFCell cell = row.createCell(c);
                        cell.setCellValue("abcd");
                    }
                }
            }

            EncryptedTempData tempData = new EncryptedTempData();
            try{
                wb.write(tempData.getOutputStream());
                save(tempData.getInputStream(), filename, password);
                System.out.println("Saved " + filename);
            }finally {
                tempData.dispose();
            }
        }finally {
            wb.close();
            //the dispose call is necessary to ensure temp files are removed
            wb.dispose();
        }
        TempFileUtils.checkTempFiles();
    }

    public static void save( InputStream inputStream,  String filename,  String pwd)
            throws InvalidFormatException, IOException, GeneralSecurityException {

        try (POIFSFileSystem fs = new POIFSFileSystem();
             OPCPackage opc = OPCPackage.open(inputStream);
             FileOutputStream fos = new FileOutputStream(filename)) {
             EncryptionInfo info = new EncryptionInfo(EncryptionMode.agile);
             Encryptor enc = Encryptor.getInstance(info);
            enc.confirmPassword(pwd);
            opc.save(enc.getDataStream(fs));
            fs.writeFilesystem(fos);
        }finally {
            IOUtils.closeQuietly(inputStream);
        }
    }
}
