package com.iohk.jormungandrwallet;

public class Settings {
    static {
        System.loadLibrary("wallet_jni");
    }

    // new can't be used as the name for this
    public native static long build(LinearFees fees, Discrimination discrimination, byte[] block0_hash);

    public native static LinearFees fees(long settings);

    public native static Discrimination discrimination(long settings);

    public native static byte[] block0Hash(long settings);

    public native static void delete(long settings);

    public static enum Discrimination {
        PRODUCTION, TEST;

        // JNI helper to make it easier to compare and avoid lookups
        byte discriminant() {
            if (this == PRODUCTION) {
                return 0;
            }
            else if (this == TEST) {
                return 1;
            }
            else {
                throw new RuntimeException("unhandled variant");
            }
        }
    }

    public static class LinearFees {
        public long constant;
        public long coefficient;
        public long certificate;
        public PerCertificateFee perCertificateFee;
        public PerVoteCertificateFee perVoteCertificateFee;

        public LinearFees(long constant, long coefficient, long certificate, PerCertificateFee perCertificateFee, PerVoteCertificateFee perVoteCertificateFee) {
            this.constant = constant;
            this.coefficient = coefficient;
            this.certificate = certificate;
            this.perCertificateFee = perCertificateFee;
            this.perVoteCertificateFee = perVoteCertificateFee;
        }

        // a single constructor is way easier to invoke from jni (and a bit
        // faster, probably)
        LinearFees(long constant, long coefficient, long certificate,
        long poolRegistration, long stakeDelegation, long
        ownerStakeDelegation, long votePlan, long voteCast) {
            this.constant = constant;
            this.coefficient = coefficient;
            this.certificate = certificate;
            this.perCertificateFee = new PerCertificateFee(poolRegistration, stakeDelegation, ownerStakeDelegation);
            this.perVoteCertificateFee = new PerVoteCertificateFee(votePlan, voteCast);
        }

        // jni helper to avoid getting all the fields one by one, which is
        // expensive and cumbersome
        long[] pack() {
            final long[] result = {
                this.constant, 
                this.coefficient,
                this.certificate,
                this.perCertificateFee.certificatePoolRegistration,
                this.perCertificateFee.certificateStakeDelegation,
                this.perCertificateFee.certificateOwnerStakeDelegation,
                this.perVoteCertificateFee.certificateVotePlan,
                this.perVoteCertificateFee.certificateVoteCast,
            };

            return result;
        }
    }

    public static class PerCertificateFee {
        public long certificatePoolRegistration;
        public long certificateStakeDelegation;
        public long certificateOwnerStakeDelegation;

        public PerCertificateFee(long registration, long stakeDelegation, long ownerStakeDelegation) {
            this.certificatePoolRegistration = registration;
            this.certificateStakeDelegation = stakeDelegation;
            this.certificateOwnerStakeDelegation = ownerStakeDelegation;
        }
    }

    public static class PerVoteCertificateFee {
        public long certificateVotePlan;
        public long certificateVoteCast;

        public PerVoteCertificateFee(long votePlan, long voteCast) {
            certificateVotePlan = votePlan;
            certificateVoteCast = voteCast;
        }
    }
}