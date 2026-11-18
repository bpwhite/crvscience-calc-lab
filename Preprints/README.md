# Preprints for Calc-Lab (`crvscience-calc-lab`)

**Author:** Bryan P. White (ORCID: [0000-0003-4191-3511](https://orcid.org/0000-0003-4191-3511))  
**Hub:** https://crvscience.com/apps

This directory contains the self-contained LaTeX source (`.tex`) and single-column PDF (`.pdf`) editions for both:

1. **Companion Application Preprint (`APP-PREPRINT-03`)**
   - **Title:** Calc-Lab: Interactive Calculus, Linear Algebra, and Differential Equation Solver Workbench
   - **Files:** `calc-lab-scientific-calculus-ode.tex`, `calc-lab-scientific-calculus-ode.pdf`
   - **Canonical Zenodo DOI:** https://doi.org/10.5281/zenodo.24222372
   - **Figshare Mirror:** https://figshare.com/articles/preprint/Calc-Lab_Interactive_Calculus_Linear_Algebra_and_Differential_Equation_Solver_Workbench/35013748
   - **Web Preprint:** https://crvscience.com/apps/preprints/calc-lab-scientific-calculus-ode

2. **CRV Science Public Apps Network Overview (`APP-PREPRINT-01`)**
   - **Title:** The CRV Science App Network: Eighteen Open-Source Rust Programs for Science, Data, and Simulation
   - **Files:** `crv-science-public-apps-network.tex`, `crv-science-public-apps-network.pdf`
   - **Canonical Zenodo DOI:** https://doi.org/10.5281/zenodo.24222398
   - **Figshare Mirror:** https://figshare.com/articles/preprint/The_CRV_Science_App_Network_Seventeen_Open-Source_Rust_Programs_for_Science_Data_and_Simulation/35013736
   - **Web Preprint:** https://crvscience.com/apps/preprints/crv-science-public-apps-network

## Compiling Standalone LaTeX

Both `.tex` files inline their bibliography so each compiles standalone with standard `pdflatex`:

```bash
pdflatex calc-lab-scientific-calculus-ode.tex
pdflatex crv-science-public-apps-network.tex
```
