package io.matt.struct.briage.fileupload;

public interface FileUploadExecutor {

    Object uploadFile(String path);

    boolean checkFile(Object object);
}
