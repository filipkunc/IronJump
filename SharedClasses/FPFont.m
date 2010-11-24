//
//  FPFont.m
//  IronJump
//
//  Created by Filip Kunc on 5/10/10.
//  Copyright (c) 2010 Filip Kunc. All rights reserved.
//

#import "FPFont.h"

@implementation FPFont

@synthesize spacing;

- (id)initWithFile:(NSString *)fileName tileSize:(int)aTileSize spacing:(float)aSpacing
{
	self = [super initWithFile:fileName convertToAlpha:YES tileSize:aTileSize];
	if (self)
	{
		spacing = aSpacing;
	}
	return self;
}

- (void)drawText:(NSString *)text atPoint:(CGPoint)pt
{
	if (text == nil)
		return;
	
#if TARGET_OS_IPHONE
	pt.x -= 80.0f;
	pt.y += 80.0f;
#endif
	
	GLfloat texCoordX = 1.0f / self.horizontalTileCount;
	GLfloat texCoordY = 1.0f / self.verticalTileCount;	
	int length = [text length];
		
	FPAtlasVertex *vertexPtr = globalVertexBuffer;
	int verticesUsed = 0;
	
	for (int i = 0; i < length; i++)
	{
		int character = [text characterAtIndex:i];
		if (character != ' ')
		{
			int y = character / 16;
			int x = character - (y * 16);
			
			const FPAtlasVertex vertices[] = 
			{
				{ pt.x,				pt.y,				x * texCoordX,			y * texCoordY, },			// 0
				{ pt.x + tileSize,	pt.y,				(x + 1) * texCoordX,	y *	texCoordY, },			// 1
				{ pt.x,				pt.y + tileSize,	x * texCoordX,			(y + 1) * texCoordY, },		// 2
				
				{ pt.x,				pt.y + tileSize,	x * texCoordX,			(y + 1) * texCoordY, },		// 2
				{ pt.x + tileSize,	pt.y + tileSize,	(x + 1) * texCoordX,	(y + 1) * texCoordY, },		// 3
				{ pt.x + tileSize,	pt.y,				(x + 1) * texCoordX,	y *	texCoordY, },			// 1
			};
			
			verticesUsed += 6;
			if (verticesUsed > kMaxVertices)
			{
				@throw [NSException exceptionWithName:@"TextureDrawingFailed" reason:@"verticesUsed >= kMaxVertices" userInfo:nil];
			}
			
			memcpy(vertexPtr, vertices, sizeof(vertices));
			vertexPtr += 6;
		}
		pt.x += spacing;
	}
	
	if (verticesUsed > 0)
	{
		glEnable(GL_TEXTURE_2D);
		glBindTexture(GL_TEXTURE_2D, [texture textureID]);
		glVertexPointer(2, GL_FLOAT, sizeof(FPAtlasVertex), &globalVertexBuffer->x);
		glEnableClientState(GL_VERTEX_ARRAY);
		glTexCoordPointer(2, GL_FLOAT, sizeof(FPAtlasVertex), &globalVertexBuffer->s);
		glEnableClientState(GL_TEXTURE_COORD_ARRAY);
		glDrawArrays(GL_TRIANGLES, 0, verticesUsed);
	}
}


@end
