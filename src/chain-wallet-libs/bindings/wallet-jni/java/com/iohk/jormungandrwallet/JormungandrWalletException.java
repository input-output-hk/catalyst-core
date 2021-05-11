package com.iohk.jormungandrwallet;

import com.iohk.jormungandrwallet.*;
/**
 * 
 * 
 */
public class JormungandrWalletException extends Exception {

	private final ErrorCode code;

	public JormungandrWalletException(ErrorCode code) {
		super();
		this.code = code;
	}

	public JormungandrWalletException(String message, Throwable cause, ErrorCode code) {
		super(message, cause);
		this.code = code;
	}

	public JormungandrWalletException(String message, ErrorCode code) {
		super(message);
		this.code = code;
	}

	public JormungandrWalletException(Throwable cause, ErrorCode code) {
		super(cause);
		this.code = code;
	}
	
	public ErrorCode getCode() {
		return this.code;
	}
}