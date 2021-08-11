package com.iohk.jormungandrwallet;

public class Time {
    public static class BlockDate {
        public long epoch;
        public long slot;

        public BlockDate(long epoch, long slot) {
            this.epoch = epoch;
            this.slot = slot;
        }
    }

    public native static BlockDate blockDateFromSystemTime(long settings, long date);
    public native static BlockDate maxExpirationDate(long settings, long currentDate);
}
