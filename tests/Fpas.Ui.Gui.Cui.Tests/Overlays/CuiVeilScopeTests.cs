namespace Fpas.Ui.Gui.Cui.Tests.Overlays;

/// <summary>Checks covered native body scopes and separately admitted composed content.</summary>
public sealed class CuiVeilScopeTests
{
    /// <summary>Nested covered veils block child focus and leave later siblings enabled.</summary>
    /// <param name="renderer">Renderer used for native presentation.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void NestedVeilsPreserveCoveredBodyScopes(CuiRenderer renderer)
    {
        Verify(outerShown: false, innerShown: true);
        Verify(outerShown: true, innerShown: false);
        Verify(outerShown: true, innerShown: true);

        void Verify(bool outerShown, bool innerShown)
        {
            using var window = new CuiWindow("Nested veil scopes", 480, 320, renderer, continuous: true);
            window.SetReducedMotion(true);
            Draw(focusSibling: false);
            Assert.Equal(outerShown && innerShown, window.ControlFocused("child"));
            Draw(focusSibling: true);
            Assert.True(window.ControlFocused("sibling"));

            void Draw(bool focusSibling)
            {
                Assert.True(window.NextFrame());
                window.BeginGroup(false, 0, 8, 0, false);
                // Covered chrome may coexist with independently admitted modal content.
                int outer = window.VeilBegin("outer", outerShown, null, null, 0, 0, 0, 0, 0);
                int inner = window.VeilBegin("inner", innerShown, null, null, 0, 0, 0, 0, 0);
                Assert.Equal(outerShown, (outer & 1) != 0);
                Assert.Equal(innerShown, (inner & 1) != 0);
                Assert.False(window.Button("child", "Child", 0, 0, true, false, false,
                    !focusSibling, "", "", "", "", false, -1, 0));
                window.TrackLastControl("child");
                window.VeilEnd();
                window.VeilEnd();
                Assert.False(window.Button("sibling", "Sibling", 0, 0, true, false, false,
                    focusSibling, "", "", "", "", false, -1, 0));
                window.TrackLastControl("sibling");
                window.EndGroup();
                window.Present();
            }
        }
    }
}
