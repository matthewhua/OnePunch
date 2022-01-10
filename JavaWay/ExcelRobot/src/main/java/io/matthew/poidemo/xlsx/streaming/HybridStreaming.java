package io.matthew.poidemo.xlsx.streaming;


import org.apache.poi.xssf.eventusermodel.ReadOnlySharedStringsTable;
import org.apache.poi.xssf.eventusermodel.XSSFSheetXMLHandler;
import org.apache.poi.xssf.usermodel.XSSFComment;
import org.apache.poi.xssf.usermodel.XSSFSheet;
import org.apache.poi.xssf.usermodel.XSSFWorkbook;
import org.openxmlformats.schemas.spreadsheetml.x2006.main.CTSheet;
import org.xml.sax.SAXException;
import org.apache.poi.xssf.eventusermodel.XSSFSheetXMLHandler.SheetContentsHandler;
import java.io.FileInputStream;
import java.io.FileNotFoundException;
import java.io.IOException;
import java.io.InputStream;
import java.util.Map;

/**
 * This demonstrates how a hybrid approach to workbook read can be taken, using
 * a mix of traditional XSSF and streaming one particular worksheet (perhaps one
 * which is too big for the ordinary DOM parse).
 */
public class HybridStreaming {

    private static final String SHEET_TO_STREAM = "large sheet";

    public static void main(String[] args) throws IOException, SAXException {
        try (InputStream sourceBytes = new FileInputStream("DeferredGeneration.xlsx")) {
            final XSSFWorkbook workbook = new XSSFWorkbook(sourceBytes){
                /**
                 * Avoid DOM parse of large sheet
                 */
                @Override
                public void parseSheet(Map<String, XSSFSheet> shIdMap, CTSheet ctSheet) {
                    if (!SHEET_TO_STREAM.equals(ctSheet.getName())) {
                        super.parseSheet(shIdMap, ctSheet);
                    }
                }
            };

            // Having avoided a DOM-based parse of the sheet, we can stream it instead.
            final ReadOnlySharedStringsTable strings = new ReadOnlySharedStringsTable(workbook.getPackage());
            new XSSFSheetXMLHandler(workbook.getStylesSource(), strings, createSheetContentsHandler(), false);
            workbook.close();

        }
    }


    private static SheetContentsHandler createSheetContentsHandler() {
        return new SheetContentsHandler() {

            @Override
            public void startRow(int rowNum) {
            }

            @Override
            public void endRow(int rowNum) {
            }

            @Override
            public void cell(String cellReference, String formattedValue, XSSFComment comment) {
            }
        };
    }

}
