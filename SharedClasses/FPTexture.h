//
//  FPTexture.h
//  SpriteLib
//
//  Created by Filip Kunc on 4/12/10.
//  For license see LICENSE.TXT
//

#import "FPGraphics.h"

#if TARGET_OS_IPHONE
CGImageRef LoadImage(NSString *fileName);
#endif

void CreateTexture(GLubyte *data, int components, GLuint *textureID, int width, int height, BOOL convertToAlpha);

@interface FPTexture : NSObject 
{
	GLuint textureID;
	int width;
	int height;
}

@property (readonly, assign) GLuint textureID;
@property (readonly, assign) int width;
@property (readonly, assign) int height;

- (id)initWithFile:(NSString *)fileName convertToAlpha:(BOOL)convertToAlpha;
- (void)draw;
- (void)drawAtPoint:(CGPoint)point;
- (void)drawAtPoint:(CGPoint)point widthSegments:(int)widthSegments heightSegments:(int)heightSegments;

@end
