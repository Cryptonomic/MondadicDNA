from nada_dsl import *
import nada_numpy as na

# Checks based on SNP rs1815739 whether someone is a sprinter or an endurance athelete
# Genotype TT indicates endurance athlete
# We arbitrarily assume sprinter otherwise
# This program returns 0 if sprinter, 1 if endurance athlete
# This is a counterpart to muscle-perform.py which operates on tabular data rather than atomic secrets.
def nada_main():

    target_snp_1 = Integer(1815739)
    target_genotype_1 = Integer(9)

    party1 = Party(name="Party1")

    gene_data = na.array((1, 4), party1, "gene_data", SecretInteger)
    boolean_mask = gene_data[:, 0] == target_snp_1
    actual_genotype_1 = boolean_mask.dot(gene_data[:, 3])
    is_successful = actual_genotype_1 == target_genotype_1

    return [Output(is_successful, "Result", party1)]
