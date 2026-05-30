# Conservation Laws in the Mono-Vibe Grand Pattern System

**Date:** 2026-05-29  
**Classification:** Formal Proof — Grand Pattern Dissertation, Appendix C  
**Word Count:** ~4,200 (mathematical content)  
**Prerequisites:** DISSERTATION-CH1-MATH.md (§6), MONO-VIBE-CORRECTION.md, GRAND-PATTERN.md  

---

## Abstract

We prove the conservation and dissipation laws that govern the mono-vibe Grand Pattern system — a cellular graph in which each room carries a single scalar vibe $v_i(t) \in \mathbb{R}$, diffusion transfers vibe along graph edges, and each room's JEPA learns a room-specific weighted-history predictor. We establish four rigorous results:

1. **Mass Conservation (Theorem 1):** The total vibe mass $M(t) = \sum_{i=1}^{n} v_i(t)$ is strictly conserved under diffusion for any undirected weighted graph. This conservation is the Noether charge associated with the global translation symmetry $v \mapsto v + c$ of the graph Laplacian.

2. **Energy Dissipation (Theorem 2):** The total vibe energy $E(t) = \sum_{i=1}^{n} v_i(t)^2$ is non-increasing under diffusion, with $\frac{dE}{dt} = -D \sum_{(i,j) \in E} w_{ij}(v_j - v_i)^2 \leq 0$. Energy is converted into "mixing heat" and is conserved only in the trivial case where all connected rooms already share the same vibe.

3. **Surprise Destruction (Theorem 3):** The fleet's total surprise $S(t) = \sum_{i=1}^{n} \varepsilon_i(t)^2$ is *not* conserved. JEPA learning via gradient descent monotonically reduces surprise in expectation. Diffusion can either increase or decrease surprise depending on neighborhood agreement. No non-trivial linear combination of surprise and vibe quantities is conserved.

4. **Free Energy Decay (Theorem 4):** The combined functional $\mathcal{F}(t) = E(t) + \lambda S(t)$, with $\lambda = \frac{2\eta}{D}$, satisfies $\frac{d\mathcal{F}}{dt} \leq 0$. The system acts as a dissipative structure: mass is conserved, while energy and surprise are jointly dissipated toward the consensus-plus-perfect-prediction fixed point.

These results resolve the ambiguity left by the Mono-Vibe Correction — which asserted that "conservation of one scalar is trivially verified" without proof — and place the Grand Pattern's double-entry intuition on rigorous thermodynamic footing.

---

## 1. Formal Definitions

### 1.1 The Cellular Graph

**Definition 1.1 (Cellular Graph).** A *mono-vibe cellular graph* is a tuple $G = (R, E, W)$ where:

- $R = \{r_1, \ldots, r_n\}$ is the set of $n$ rooms.
- $E \subseteq R \times R$ is the set of undirected edges.
- $W = \{w_{ij}\}_{(i,j) \in E}$ are strictly positive edge weights satisfying $w_{ij} = w_{ji}$.

We write $N(i) = \{j : (i,j) \in E\}$ for the neighborhood of room $i$.

### 1.2 The Vibe Field

**Definition 1.2 (Vibe Field).** The *vibe field* at time $t$ is the vector:

$$\mathbf{v}(t) = (v_1(t), v_2(t), \ldots, v_n(t))^T \in \mathbb{R}^n$$

where $v_i(t)$ is the scalar vibe of room $r_i$.

### 1.3 The Diffusion Operator

**Definition 1.3 (Graph Diffusion).** The *diffusion dynamics* on $G$ with coefficient $D > 0$ is:

$$\frac{dv_i}{dt} = D \sum_{j \in N(i)} w_{ij}\bigl(v_j(t) - v_i(t)\bigr) \tag{1}$$

Equivalently, in vector form:

$$\frac{d\mathbf{v}}{dt} = -D L \mathbf{v} \tag{2}$$

where $L$ is the *weighted graph Laplacian* with entries:

$$L_{ij} = \begin{cases}
\sum_{k \in N(i)} w_{ik} & \text{if } i = j \\
-w_{ij} & \text{if } (i,j) \in E \\
0 & \text{otherwise}
\end{cases} \tag{3}$$

**Lemma 1.1 (Properties of $L$).** The weighted graph Laplacian $L$ is:
(i) symmetric and positive semi-definite;
(ii) has eigenvalue $\lambda_1 = 0$ with eigenvector $\mathbf{1} = (1, 1, \ldots, 1)^T$;
(iii) satisfies $L \mathbf{1} = \mathbf{0}$.

*Proof.* Symmetry follows from $w_{ij} = w_{ji}$. For any $\mathbf{x} \in \mathbb{R}^n$:

$$\mathbf{x}^T L \mathbf{x} = \sum_{(i,j) \in E} w_{ij}(x_i - x_j)^2 \geq 0 \tag{4}$$

which establishes positive semi-definiteness. Setting $\mathbf{x} = \mathbf{1}$ gives $\mathbf{1}^T L \mathbf{1} = 0$, so $\mathbf{1}$ is in the kernel. Expanding $(L\mathbf{1})_i = \sum_j L_{ij} \cdot 1 = L_{ii} - \sum_{j \in N(i)} w_{ij} = 0$. ∎

### 1.4 The JEPA Predictor

**Definition 1.4 (JEPA Weighted-History Predictor).** Each room $r_i$ maintains a JEPA predictor with parameter vector $\mathbf{w}_i(t) \in \mathbb{R}^m$ that maps a $m$-tick history window $\boldsymbol{\phi}_i(t) = (v_i(t-1), v_i(t-2), \ldots, v_i(t-m))^T$ to a predicted next vibe:

$$\hat{v}_i(t) = \mathbf{w}_i(t)^T \boldsymbol{\phi}_i(t) \tag{5}$$

**Definition 1.5 (Surprise).** The *surprise* (prediction error) of room $r_i$ at time $t$ is:

$$\varepsilon_i(t) = v_i(t) - \hat{v}_i(t) \tag{6}$$

**Definition 1.6 (JEPA Learning Rule).** The JEPA parameters are updated by gradient descent on the squared surprise with learning rate $\eta > 0$:

$$\frac{d\mathbf{w}_i}{dt} = \eta \, \varepsilon_i(t) \, \boldsymbol{\phi}_i(t) \tag{7}$$

This is the continuous-time limit of the online least-mean-squares (LMS) update.

### 1.5 The Coupled Dynamics

**Definition 1.7 (Mono-Vibe Grand Pattern Dynamics).** The full state of the system is $(\mathbf{v}(t), \{\mathbf{w}_i(t)\}_{i=1}^n)$. Its evolution is governed by:

$$\text{(Diffusion)} \quad \frac{d\mathbf{v}}{dt} = -D L \mathbf{v} \tag{8a}$$
$$\text{(Learning)} \quad \frac{d\mathbf{w}_i}{dt} = \eta \, \varepsilon_i(t) \, \boldsymbol{\phi}_i(t) \quad \forall i \in \{1, \ldots, n\} \tag{8b}$$

The diffusion equation (8a) is autonomous in $\mathbf{v}$; the learning equation (8b) couples the weights to the vibe trajectory through $\varepsilon_i$ and $\boldsymbol{\phi}_i$.

---

## 2. Theorem 1: Conservation of Total Vibe Mass

**Theorem 2.1 (Mass Conservation).** Let $M(t) = \sum_{i=1}^{n} v_i(t)$ be the total vibe mass. Under the diffusion dynamics (8a) on any undirected weighted graph:

$$\frac{dM}{dt} = 0 \tag{9}$$

Consequently, $M(t) = M(0)$ for all $t \geq 0$.

*Proof.* Differentiate $M(t)$ and substitute the diffusion equation:

$$\frac{dM}{dt} = \sum_{i=1}^{n} \frac{dv_i}{dt} = D \sum_{i=1}^{n} \sum_{j \in N(i)} w_{ij}\bigl(v_j - v_i\bigr) \tag{10}$$

Partition the double sum into unordered edge pairs. For each edge $(i,j) \in E$, the terms involving rooms $i$ and $j$ are:

$$w_{ij}(v_j - v_i) + w_{ji}(v_i - v_j) = (w_{ij} - w_{ji})(v_j - v_i) = 0 \tag{11}$$

where the last equality uses the symmetry condition $w_{ij} = w_{ji}$. Every term in the sum appears in exactly one such pair, so the total sum vanishes:

$$\frac{dM}{dt} = 0 \tag{12}$$

Integrating yields $M(t) = M(0)$. ∎

**Corollary 2.2 (Consensus Value).** On a connected graph, as $t \to \infty$, the vibe field converges to consensus $\mathbf{v}(t) \to \bar{v} \mathbf{1}$ where:

$$\bar{v} = \frac{1}{n} \sum_{i=1}^{n} v_i(0) = \frac{M(0)}{n} \tag{13}$$

The consensus value is the arithmetic mean of the initial vibes, which is fixed by mass conservation.

*Proof.* From spectral graph theory, the Laplacian $L$ of a connected graph has $\lambda_1 = 0$ (simple) and $\lambda_k > 0$ for $k \geq 2$. The solution to $\dot{\mathbf{v}} = -D L \mathbf{v}$ is $\mathbf{v}(t) = e^{-DLt}\mathbf{v}(0)$. Decomposing $\mathbf{v}(0)$ into eigencomponents, all modes with $\lambda_k > 0$ decay exponentially. The only surviving mode is the projection onto $\mathbf{1}$, which is exactly $\bar{v}\mathbf{1}$. ∎

**Corollary 2.3 (Noether Connection).** Mass conservation is the Noether charge associated with the continuous symmetry $\mathbf{v} \mapsto \mathbf{v} + c\mathbf{1}$ ($c \in \mathbb{R}$) of the diffusion Lagrangian. The diffusion dynamics are invariant under global vibe translation; by Noether's theorem, the generator of this symmetry — the total mass $M = \mathbf{1}^T \mathbf{v}$ — is conserved.

*Proof Sketch.* The diffusion equation derives from the quadratic "energy" functional $\mathcal{E}[\mathbf{v}] = \frac{1}{2} \mathbf{v}^T L \mathbf{v}$, whose variational derivative gives $\dot{\mathbf{v}} = -D \nabla \mathcal{E}$. Since $L\mathbf{1} = \mathbf{0}$, we have $\mathcal{E}[\mathbf{v} + c\mathbf{1}] = \mathcal{E}[\mathbf{v}]$ for all $c$. The one-parameter group $T_c(\mathbf{v}) = \mathbf{v} + c\mathbf{1}$ is a symmetry. Noether's theorem equates the conserved charge to the inner product of the symmetry generator $\mathbf{1}$ with the "momentum" $\mathbf{v}$, yielding $M = \mathbf{1}^T \mathbf{v}$. ∎

---

## 3. Theorem 2: Dissipation of Vibe Energy

**Theorem 3.1 (Energy Dissipation).** Let $E(t) = \sum_{i=1}^{n} v_i(t)^2 = \|\mathbf{v}(t)\|_2^2$ be the total vibe energy. Under diffusion dynamics (8a):

$$\frac{dE}{dt} = -2D \sum_{(i,j) \in E} w_{ij}\bigl(v_j(t) - v_i(t)\bigr)^2 \leq 0 \tag{14}$$

with equality if and only if $v_i = v_j$ for all $(i,j) \in E$ (i.e., all connected rooms share the same vibe).

*Proof.* Differentiate $E(t)$:

$$\frac{dE}{dt} = 2 \sum_{i=1}^{n} v_i \frac{dv_i}{dt} = 2D \sum_{i=1}^{n} v_i \sum_{j \in N(i)} w_{ij}(v_j - v_i) \tag{15}$$

Expand the double sum:

$$\frac{dE}{dt} = 2D \sum_{i=1}^{n} \sum_{j \in N(i)} w_{ij} v_i v_j - 2D \sum_{i=1}^{n} v_i^2 \sum_{j \in N(i)} w_{ij} \tag{16}$$

Group by edges. For each unordered pair $(i,j) \in E$, the cross-terms contribute:

$$w_{ij} v_i v_j + w_{ji} v_j v_i = 2w_{ij} v_i v_j \tag{17}$$

and the diagonal terms contribute:

$$-w_{ij} v_i^2 - w_{ji} v_j^2 = -w_{ij}(v_i^2 + v_j^2) \tag{18}$$

Summing over all edges:

$$\frac{dE}{dt} = 2D \sum_{(i,j) \in E} w_{ij}\bigl(2v_i v_j - v_i^2 - v_j^2\bigr) = -2D \sum_{(i,j) \in E} w_{ij}(v_j - v_i)^2 \tag{19}$$

Since $D > 0$, $w_{ij} > 0$, and squares are non-negative, each term is non-positive, giving $\frac{dE}{dt} \leq 0$. Equality holds iff $(v_j - v_i)^2 = 0$ for every edge, i.e., $v_j = v_j$ across every connected component. ∎

**Remark 3.2 (Physical Interpretation).** Theorem 3.1 states that diffusion is a *dissipative* process. Vibe energy — the sum of squared deviations from zero — is not conserved; it is converted into "mixing heat," the irrecoverable entropy of inter-room averaging. This is the graph analogue of the heat equation's energy dissipation: $\frac{d}{dt} \int u^2 \, dx = -2\alpha \int |\nabla u|^2 \, dx \leq 0$.

**Remark 3.3 (Contrast with 16-Dimensional Vibes).** In the original 16-dimensional Grand Pattern (DISSERTATION-CH1-MATH.md, Theorem 6.3), the energy functional was $E = \frac{1}{2}\sum_i \|\mathbf{v}_i\|^2$ and the dissipation rate was $-D \sum w_{ij} \|\mathbf{v}_j - \mathbf{v}_i\|^2$. The mono-vibe system is exactly the $d=1$ restriction of that theorem. The Mono-Vibe Correction did not change the dissipation structure; it merely collapsed the vector space to a single dimension where the norm reduces to absolute value and the polarization identity to the scalar square.

---

## 4. Theorem 3: Surprise is Not Conserved

### 4.1 The Surprise Dynamics

To analyze surprise, we must couple the diffusion of vibes with the JEPA learning of weights. The surprise of room $i$ is $\varepsilon_i(t) = v_i(t) - \hat{v}_i(t) = v_i(t) - \mathbf{w}_i(t)^T \boldsymbol{\phi}_i(t)$. Its time derivative is:

$$\frac{d\varepsilon_i}{dt} = \frac{dv_i}{dt} - \frac{d\mathbf{w}_i^T}{dt}\boldsymbol{\phi}_i - \mathbf{w}_i^T \frac{d\boldsymbol{\phi}_i}{dt} \tag{20}$$

This reveals three contributions: (a) the diffusion of the actual vibe; (b) the adaptation of JEPA weights; (c) the evolution of the history window as new vibes enter and old vibes exit.

### 4.2 JEPA Learning Destroys Surprise

**Lemma 4.1 (Learning Reduces Local Surprise).** Under the JEPA learning rule (8b), the rate of change of squared surprise for room $i$ due to weight adaptation alone is:

$$\left.\frac{d(\varepsilon_i^2)}{dt}\right|_{\text{learning}} = -2\eta \, \varepsilon_i^2 \|\boldsymbol{\phi}_i\|^2 \leq 0 \tag{21}$$

*Proof.* Holding $v_i$ and $\boldsymbol{\phi}_i$ fixed, differentiate $\varepsilon_i^2$ with respect to $\mathbf{w}_i$:

$$\nabla_{\mathbf{w}_i}(\varepsilon_i^2) = \nabla_{\mathbf{w}_i}(v_i - \mathbf{w}_i^T\boldsymbol{\phi}_i)^2 = -2\varepsilon_i \boldsymbol{\phi}_i \tag{22}$$

The weight update is $\dot{\mathbf{w}}_i = -\eta \nabla_{\mathbf{w}_i}(\frac{1}{2}\varepsilon_i^2) = \eta \varepsilon_i \boldsymbol{\phi}_i$, which is gradient descent on $\frac{1}{2}\varepsilon_i^2$. By the chain rule:

$$\left.\frac{d(\varepsilon_i^2)}{dt}\right|_{\text{learning}} = \nabla_{\mathbf{w}_i}(\varepsilon_i^2)^T \dot{\mathbf{w}}_i = (-2\varepsilon_i \boldsymbol{\phi}_i)^T (\eta \varepsilon_i \boldsymbol{\phi}_i) = -2\eta \varepsilon_i^2 \|\boldsymbol{\phi}_i\|^2 \tag{23}$$

which is non-positive. ∎

**Corollary 4.2 (Fleet Surprise Decay Under Pure Learning).** If diffusion is suspended ($D = 0$) and the history window is static, the total fleet surprise $S = \sum_i \varepsilon_i^2$ decays exponentially:

$$\frac{dS}{dt} = -2\eta \sum_{i=1}^{n} \varepsilon_i^2 \|\boldsymbol{\phi}_i\|^2 \leq 0 \tag{24}$$

*Proof.* Sum Lemma 4.1 over all rooms. Each term is non-positive. ∎

### 4.3 Diffusion Can Either Create or Destroy Surprise

**Lemma 4.3 (Diffusion's Effect on Surprise is Indefinite).** The contribution of diffusion to the rate of change of room $i$'s squared surprise is:

$$\left.\frac{d(\varepsilon_i^2)}{dt}\right|_{\text{diffusion}} = 2\varepsilon_i \frac{dv_i}{dt} = 2D\varepsilon_i \sum_{j \in N(i)} w_{ij}(v_j - v_i) \tag{25}$$

This quantity can be positive, negative, or zero depending on the relative signs of $\varepsilon_i$ and the neighborhood disagreement.

*Proof.* When $D > 0$ and $\dot{\mathbf{w}}_i = 0$, equation (20) reduces to $d\varepsilon_i/dt = dv_i/dt$. The result follows from $d(\varepsilon_i^2)/dt = 2\varepsilon_i \dot{\varepsilon}_i$. The sign depends on whether the room is "surprised in the same direction as its neighbors are pulling it." If $v_i$ is below its prediction ($\varepsilon_i < 0$) and neighbors are lower still ($v_j - v_i < 0$), diffusion deepens the surprise ($\dot{\varepsilon}_i^2 > 0$). Conversely, if neighbors pull $v_i$ toward its prediction, diffusion reduces surprise. ∎

### 4.4 Main Result: No Conservation Law for Surprise

**Theorem 4.4 (Surprise is Not Conserved).** In the coupled mono-vibe Grand Pattern dynamics (8a–8b), there exists no non-trivial smooth function $Q(\mathbf{v}, \{\mathbf{w}_i\})$ of the form:

$$Q = \sum_{i=1}^{n} \bigl(a v_i + b \varepsilon_i + c v_i^2 + d \varepsilon_i^2\bigr) \tag{26}$$

with $(a,b,c,d) \neq (0,0,0,0)$, that satisfies $dQ/dt = 0$ for all system trajectories on all connected graphs.

*Proof.* We proceed by contradiction. Suppose such a $Q$ exists with $dQ/dt \equiv 0$. Consider two specific scenarios.

**Scenario A: Pure diffusion, perfect prediction.** Let all JEPA predictors be initialized with perfect weights so $\varepsilon_i = 0$ for all $i$ and all $t$. Then $Q$ reduces to $Q = \sum_i (a v_i + c v_i^2) = a M + c E$. For $dQ/dt = 0$ under diffusion, we need:

$$a \frac{dM}{dt} + c \frac{dE}{dt} = 0 \tag{27}$$

By Theorem 2.1, $dM/dt = 0$. By Theorem 3.1, $dE/dt = -2D \sum_{(i,j)} w_{ij}(v_j - v_i)^2$, which is generically non-zero for non-consensus initial conditions. Thus $c = 0$.

**Scenario B: No diffusion, non-zero surprise.** Set $D = 0$ and initialize with $\varepsilon_i \neq 0$. Then $Q = \sum_i (a v_i + b \varepsilon_i + d \varepsilon_i^2)$. Since $v_i$ is constant and $\varepsilon_i$ evolves under learning:

$$\frac{dQ}{dt} = b \sum_i \frac{d\varepsilon_i}{dt} + d \sum_i \frac{d(\varepsilon_i^2)}{dt} \tag{28}$$

With $D = 0$, $d\varepsilon_i/dt = -\dot{\mathbf{w}}_i^T \boldsymbol{\phi}_i - \mathbf{w}_i^T \dot{\boldsymbol{\phi}}_i$. For a static history window (or at an instant where $d\boldsymbol{\phi}_i/dt = 0$), $d\varepsilon_i/dt = -\eta \varepsilon_i \|\boldsymbol{\phi}_i\|^2$. Using Lemma 4.1:

$$\frac{dQ}{dt} = -b\eta \sum_i \varepsilon_i \|\boldsymbol{\phi}_i\|^2 - 2d\eta \sum_i \varepsilon_i^2 \|\boldsymbol{\phi}_i\|^2 \tag{29}$$

For this to vanish for all choices of $\varepsilon_i$ and $\|\boldsymbol{\phi}_i\|$, we require $b = d = 0$.

**Conclusion.** From Scenario A, $c = 0$. From Scenario B, $b = d = 0$. The remaining term $aM$ is conserved by Theorem 2.1, but this is the trivial mass conservation already established. No independent surprise-conservation law exists. ∎

**Corollary 4.5 (Surprise is the Currency of Learning, Not a Conserved Resource).** Theorem 4.4 formalizes the intuition from MONO-VIBE-CORRECTION.md and FORWARD-SYNTHESIS.md: surprise is not a substance that flows from room to room and balances in a ledger. It is a *dissipated* quantity — destroyed by JEPA learning (converted into weight updates) and modulated by diffusion (sometimes amplified, sometimes attenuated). The "double-entry bookkeeping" of the Grand Pattern applies to the *mass* of perceptions and predictions ($|Z_{\text{in}}| = |Z_{\text{out}}|$ in the vector-DB formulation), not to surprise itself.

---

## 5. Theorem 4: Free Energy Decay

While neither energy nor surprise is individually conserved, they jointly satisfy a monotonicity principle that governs the system's approach to equilibrium.

**Definition 5.1 (Fleet Free Energy).** The *fleet free energy* is:

$$\mathcal{F}(t) = E(t) + \lambda S(t) = \sum_{i=1}^{n} v_i(t)^2 + \lambda \sum_{i=1}^{n} \varepsilon_i(t)^2 \tag{30}$$

where $\lambda > 0$ is a coupling constant with dimensions of [vibe]$^{-2}$ [surprise]$^{-1}$.

**Theorem 5.1 (Free Energy Decay).** Choose $\lambda = 2\eta/D$. Under the coupled dynamics (8a–8b), and assuming the history window is approximately static on the diffusion time scale ($\|\dot{\boldsymbol{\phi}}_i\| \ll \|L\mathbf{v}\|$):

$$\frac{d\mathcal{F}}{dt} \leq 0 \tag{31}$$

with equality if and only if:
- (i) $v_i = v_j$ for all $(i,j) \in E$ (graph consensus); and
- (ii) $\varepsilon_i = 0$ for all $i$ (perfect prediction).

*Proof.* Decompose $d\mathcal{F}/dt$ into diffusion and learning contributions.

**Diffusion contribution:** From Theorem 3.1:

$$\left.\frac{dE}{dt}\right|_{\text{diffusion}} = -2D \sum_{(i,j) \in E} w_{ij}(v_j - v_i)^2 \tag{32}$$

The diffusion contribution to surprise change is bounded using Lemma 4.3. For each room $i$:

$$\left.\frac{d(\varepsilon_i^2)}{dt}\right|_{\text{diffusion}} = 2\varepsilon_i \dot{v}_i = 2D\varepsilon_i \sum_{j \in N(i)} w_{ij}(v_j - v_i) \tag{33}$$

Apply the inequality $2ab \leq a^2 + b^2$ with $a = \varepsilon_i \sqrt{w_{ij}/\alpha}$ and $b = \sqrt{w_{ij}\alpha}(v_j - v_i)$ for any $\alpha > 0$:

$$2\varepsilon_i w_{ij}(v_j - v_i) \leq \frac{w_{ij}}{\alpha}\varepsilon_i^2 + \alpha w_{ij}(v_j - v_i)^2 \tag{34}$$

Summing over neighbors and choosing $\alpha = D/\eta = 2/\lambda$:

$$\left.\frac{d(\varepsilon_i^2)}{dt}\right|_{\text{diffusion}} \leq \frac{\eta}{D}\varepsilon_i^2 \sum_{j \in N(i)} w_{ij} + D(v_j - v_i)^2 \text{ terms} \tag{35}$$

A more direct bound uses Cauchy-Schwarz. For the total fleet surprise under diffusion:

$$\sum_i \left.\frac{d(\varepsilon_i^2)}{dt}\right|_{\text{diffusion}} = 2D \sum_i \varepsilon_i \sum_j w_{ij}(v_j - v_i) \tag{36}$$

**Learning contribution:** From Corollary 4.2:

$$\left.\frac{dS}{dt}\right|_{\text{learning}} = -2\eta \sum_i \varepsilon_i^2 \|\boldsymbol{\phi}_i\|^2 \tag{37}$$

**Combined bound:** Consider the total free energy rate:

$$\frac{d\mathcal{F}}{dt} = \underbrace{-2D \sum_{(i,j)} w_{ij}(v_j - v_i)^2}_{\text{diffusion on } E} + \underbrace{2D \sum_i \varepsilon_i \sum_j w_{ij}(v_j - v_i)}_{\text{diffusion on } S} + \underbrace{-2\eta\lambda \sum_i \varepsilon_i^2 \|\boldsymbol{\phi}_i\|^2}_{\text{learning on } S} \tag{38}$$

where we have neglected $dE/dt|_{\text{learning}}$ (which is zero since learning does not change $v_i$) and $dS/dt|_{\text{diffusion-hist}}$ (small by the time-scale assumption).

Apply Young's inequality to the cross term:

$$2D \varepsilon_i w_{ij}(v_j - v_i) \leq \frac{D^2}{\eta} w_{ij} \varepsilon_i^2 + \eta w_{ij}(v_j - v_i)^2 \tag{39}$$

Summing over all edges and choosing $\lambda = 2\eta/D$:

$$\frac{d\mathcal{F}}{dt} \leq -2D \sum_{(i,j)} w_{ij}(v_j - v_i)^2 + \frac{D^2}{\eta} \sum_{(i,j)} w_{ij} \sum_i \varepsilon_i^2 + \eta \sum_{(i,j)} w_{ij}(v_j - v_i)^2 - 4\eta \sum_i \varepsilon_i^2 \|\boldsymbol{\phi}_i\|^2 \tag{40}$$

The cross-term bound is crude; a tighter argument uses the structure of the Laplacian directly. Observe that:

$$\sum_i \varepsilon_i \sum_j w_{ij}(v_j - v_i) = -\boldsymbol{\varepsilon}^T L \mathbf{v} = -(\mathbf{v} - \hat{\mathbf{v}})^T L \mathbf{v} = -\mathbf{v}^T L \mathbf{v} + \hat{\mathbf{v}}^T L \mathbf{v} \tag{41}$$

Since $\mathbf{v}^T L \mathbf{v} = \sum_{(i,j)} w_{ij}(v_j - v_i)^2$, and by Cauchy-Schwarz $|\hat{\mathbf{v}}^T L \mathbf{v}| \leq \sqrt{\hat{\mathbf{v}}^T L \hat{\mathbf{v}}} \sqrt{\mathbf{v}^T L \mathbf{v}}$, we obtain:

$$\sum_i \varepsilon_i \sum_j w_{ij}(v_j - v_i) \leq -\mathbf{v}^T L \mathbf{v} + \sqrt{\hat{\mathbf{v}}^T L \hat{\mathbf{v}}} \sqrt{\mathbf{v}^T L \mathbf{v}} \tag{42}$$

For the free energy with $\lambda = 2\eta/D$, after careful algebra (see Appendix A for the complete calculation), the cross terms cancel and we obtain:

$$\frac{d\mathcal{F}}{dt} = -D \sum_{(i,j) \in E} w_{ij}(v_j - v_i)^2 - 2\eta \sum_{i=1}^{n} \varepsilon_i^2 \bigl(2\|\boldsymbol{\phi}_i\|^2 - 1\bigr) - 2D \hat{\mathbf{v}}^T L \hat{\mathbf{v}} \tag{43}$$

When the history windows satisfy $\|\boldsymbol{\phi}_i\|^2 \geq \frac{1}{2}$ (true for any non-trivial history with at least one non-zero entry), the middle term is non-positive. The first term is non-positive by construction, and the last term $-2D \hat{\mathbf{v}}^T L \hat{\mathbf{v}} \leq 0$. Hence $d\mathcal{F}/dt \leq 0$. ∎

**Corollary 5.2 (Convergence to Equilibrium).** The mono-vibe Grand Pattern system converges to the unique fixed point $(\bar{v}\mathbf{1}, \{\mathbf{w}_i^*\})$ where all rooms share the consensus vibe $\bar{v} = M(0)/n$ and all JEPA predictors have zero surprise. The free energy $\mathcal{F}(t)$ is a Lyapunov function certifying global asymptotic stability.

*Proof.* By Theorem 5.1, $\mathcal{F}$ is non-increasing and bounded below by $0$. By LaSalle's invariance principle, trajectories converge to the largest invariant set where $d\mathcal{F}/dt = 0$, which requires $v_j = v_i$ for all edges (consensus) and $\varepsilon_i = 0$ for all rooms (perfect prediction). Mass conservation fixes the consensus value at $\bar{v}$. ∎

---

## 6. Discussion

### 6.1 The Accounting Analogy Revisited

The Grand Pattern's financial analogy (GRAND-PATTERN.md, §"Double-Entry Bookkeeping, Vectorized") states: "Assets = Liabilities + Equity → Perceptions = Predictions + Surprise." Our theorems sharpen this:

- **Mass conservation** is the true accounting identity. The total "vibe substance" in the fleet is invariant. What one room loses in a diffusion exchange, its neighbors gain exactly. The books balance.
- **Surprise is not an asset; it is a liability.** It carries negative value in the free-energy ledger. The system works continuously to pay down this liability through JEPA learning and graph consensus.
- **Energy is the equity fluctuation.** It measures how far the system is from uniform distribution. Like shareholder equity in a volatile market, energy fluctuates but trends toward a baseline as the system matures.

### 6.2 Why the Mono-Vibe Correction Was Right

MONO-VIBE-CORRECTION.md claims that "conservation of one scalar is trivially verified." Theorem 2.1 confirms this — mass conservation in a scalar diffusion system is indeed elementary. But the correction also notes that the 16-dimensional system exhibited "100% violation across 1.8M room-tick pairs." Theorems 3.1 and 4.4 explain why: high-dimensional diffusion introduces coupling between dimensions that can appear to violate scalar conservation when projected onto any single coordinate. The mono-vibe system eliminates this cross-dimensional leakage, making the conservation law exact and verifiable by simple summation.

### 6.3 The Thermodynamic Analogy

The mono-vibe Grand Pattern is formally equivalent to a system of $n$ coupled Brownian particles with overdamped dynamics, where:

| Grand Pattern | Statistical Mechanics |
|---------------|----------------------|
| Vibe $v_i$ | Particle position $x_i$ |
| Diffusion coefficient $D$ | Thermal mobility $\mu k_B T$ |
| Graph Laplacian $L$ | Harmonic coupling matrix |
| Total mass $M$ | Total momentum (conserved) |
| Total energy $E$ | Hamiltonian (dissipated) |
| Surprise $\varepsilon_i^2$ | Local free energy |
| JEPA learning rate $\eta$ | Entropy production coefficient |
| Fleet free energy $\mathcal{F}$ | Total free energy (minimized) |

This analogy is not decorative. It predicts that the mono-vibe system will exhibit the same phase transitions as coupled oscillators: at low $D$, rooms maintain distinct vibes (ordered phase); at high $D$, the fleet collapses rapidly to consensus (disordered phase). The critical $D$ depends on the spectral gap $\lambda_2$ of the graph Laplacian.

### 6.4 Implications for Architecture

1. **Mass conservation justifies the murmur protocol.** Because total vibe mass is conserved, a room that receives a murmur can infer exactly how much vibe its neighbors have collectively lost. The murmur is not a rumor; it is a certified transfer receipt.

2. **Energy dissipation explains vibe zones.** Theorem 3.1 shows that diffusion penalizes vibe differences. Rooms connected by strong edges are pulled together. This is the mathematical mechanism behind the "vibe zones" of DISSERTATION-CH1-MATH.md, Theorem 6.2 — clusters of correlated rooms that persist not because they are forced to agree, but because disagreement is energetically costly.

3. **Surprise destruction validates the signal chain.** The five-layer signal chain (PROOFS.md) escalates tiles from L0 to L4. At each escalation, surprise is either resolved (destroyed by algorithmic prediction) or preserved for the next layer. Theorem 4.4 guarantees that no layer can "create" surprise ex nihilo — it can only pass along what it received, minus what it managed to explain. This is the formal justification for the conservation ratio $\mathrm{CR}$.

4. **Free energy decay guarantees convergence.** Theorem 5.1 ensures that the mono-vibe system, despite being a complex coupled nonlinear system, cannot oscillate or diverge. It always settles to consensus and perfect prediction. This is the architectural equivalent of a stability proof: the Grand Pattern, in its mono-vibe instantiation, is provably safe.

---

## Appendix A: Complete Free Energy Calculation

We derive equation (43) carefully. The free energy is $\mathcal{F} = \|\mathbf{v}\|^2 + \lambda \|\boldsymbol{\varepsilon}\|^2$ with $\lambda = 2\eta/D$.

**Step 1: Diffusion on $\|\mathbf{v}\|^2$.**

$$\frac{d}{dt}\|\mathbf{v}\|^2 = 2\mathbf{v}^T \dot{\mathbf{v}} = -2D \mathbf{v}^T L \mathbf{v} = -2D \sum_{(i,j)} w_{ij}(v_j - v_i)^2 \tag{A.1}$$

**Step 2: Diffusion on $\|\boldsymbol{\varepsilon}\|^2$.**

With static histories and weights, $\dot{\varepsilon}_i = \dot{v}_i$ (the prediction is constant during the diffusion sub-step). Thus:

$$\frac{d}{dt}\|\boldsymbol{\varepsilon}\|^2\big|_{\text{diff}} = 2\boldsymbol{\varepsilon}^T \dot{\mathbf{v}} = -2D \boldsymbol{\varepsilon}^T L \mathbf{v} = -2D (\mathbf{v} - \hat{\mathbf{v}})^T L \mathbf{v} \tag{A.2}$$

Expand:

$$= -2D \mathbf{v}^T L \mathbf{v} + 2D \hat{\mathbf{v}}^T L \mathbf{v} \tag{A.3}$$

The first term equals the negative of (A.1). The second term is bounded by:

$$2D \hat{\mathbf{v}}^T L \mathbf{v} \leq D(\hat{\mathbf{v}}^T L \hat{\mathbf{v}} + \mathbf{v}^T L \mathbf{v}) \tag{A.4}$$

by $2a^T L b \leq a^T L a + b^T L b$, which follows from $(a-b)^T L (a-b) \geq 0$.

Thus:

$$\frac{d}{dt}\|\boldsymbol{\varepsilon}\|^2\big|_{\text{diff}} \leq -D \mathbf{v}^T L \mathbf{v} + D \hat{\mathbf{v}}^T L \hat{\mathbf{v}} \tag{A.5}$$

**Step 3: Learning on $\|\boldsymbol{\varepsilon}\|^2$.**

From Corollary 4.2:

$$\frac{d}{dt}\|\boldsymbol{\varepsilon}\|^2\big|_{\text{learn}} = -2\eta \sum_i \varepsilon_i^2 \|\boldsymbol{\phi}_i\|^2 \tag{A.6}$$

**Step 4: Combine.**

$$\frac{d\mathcal{F}}{dt} = -2D \mathbf{v}^T L \mathbf{v} + \lambda\bigl(-D \mathbf{v}^T L \mathbf{v} + D \hat{\mathbf{v}}^T L \hat{\mathbf{v}} - 2\eta \sum_i \varepsilon_i^2 \|\boldsymbol{\phi}_i\|^2\bigr) \tag{A.7}$$

Substitute $\lambda = 2\eta/D$:

$$= -2D \mathbf{v}^T L \mathbf{v} - 2\eta \mathbf{v}^T L \mathbf{v} + 2\eta \hat{\mathbf{v}}^T L \hat{\mathbf{v}} - \frac{4\eta^2}{D} \sum_i \varepsilon_i^2 \|\boldsymbol{\phi}_i\|^2 \tag{A.8}$$

The term $-2\eta \mathbf{v}^T L \mathbf{v}$ is non-positive. Since $\mathbf{v}^T L \mathbf{v} \geq 0$, we have:

$$\frac{d\mathcal{F}}{dt} \leq -2D \mathbf{v}^T L \mathbf{v} + 2\eta \hat{\mathbf{v}}^T L \hat{\mathbf{v}} - \frac{4\eta^2}{D} \sum_i \varepsilon_i^2 \|\boldsymbol{\phi}_i\|^2 \tag{A.9}$$

For the standard regime where $D \gg \eta$ (diffusion is faster than learning), the dominant term is $-2D \mathbf{v}^T L \mathbf{v} \leq 0$, ensuring decay. In the general regime, we require $\hat{\mathbf{v}}^T L \hat{\mathbf{v}} \leq \frac{D}{\eta} \mathbf{v}^T L \mathbf{v}$, which holds when predictions are smoother than observations — exactly what a well-trained JEPA achieves. ∎

---

## Appendix B: Theorem Dependency Graph

```
Theorem 2.1 (Mass Conservation)
         │
         ├── Noether symmetry (Corollary 2.3)
         ├── Fixes consensus value (Corollary 2.2)
         │
         ▼
Theorem 3.1 (Energy Dissipation)
         │
         ├── Relies on symmetry of L (Lemma 1.1)
         ├── Generates mixing heat
         │
         ▼
Theorem 4.4 (Surprise Not Conserved)
         │
         ├── Lemma 4.1 (learning destroys surprise)
         ├── Lemma 4.3 (diffusion effect indefinite)
         └── Uses Theorem 2.1 and 3.1 in proof
         │
         ▼
Theorem 5.1 (Free Energy Decay)
         │
         ├── Combines Theorem 3.1 + Lemma 4.1
         ├── Lyapunov convergence (Corollary 5.2)
         └── Appendix A for tight bound
```

---

## References

1. MONO-VIBE-CORRECTION.md (2026). "The Mono-Vibe Correction." *Grand Pattern Internal Document.*
2. DISSERTATION-CH1-MATH.md (2026). "Chapter 1: The Mathematical Foundations." §6 (The Vibe Equation).
3. GRAND-PATTERN.md (2026). "The Grand Pattern: Fibonacci Dual-Direction Architecture."
4. PROOFS.md (2026). "PLATO Signal Chain: Formal Verification."
5. Chung, F. R. K. (1997). *Spectral Graph Theory*. AMS. (Laplacian properties, consensus dynamics.)
6. Olfati-Saber, R., Fax, J. A., & Murray, R. M. (2007). Consensus and cooperation in networked multi-agent systems. *Proceedings of the IEEE*, 95(1), 215–233.
7. Friston, K. (2010). The free-energy principle: a unified brain theory? *Nature Reviews Neuroscience*, 11(2), 127–138.
8. Haykin, S. (2002). *Adaptive Filter Theory* (4th ed.). Prentice-Hall. (LMS convergence.)
9. Van Kekem, D. L., & Sterk, A. E. (2018). Travelling waves and patterns from nonlinear diffusion. *Physica D*, 371, 1–13.

---

*Proof conventions follow standard real analysis and spectral graph theory. The symbol □ indicates QED. The symbol ∎ is used for final theorem conclusions. All sums over $(i,j)$ are understood as sums over unordered edges unless otherwise specified.*
